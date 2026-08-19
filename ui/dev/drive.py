#!/usr/bin/env python3
"""Drive the real desktop app from a script, for development.

The shell can be rendered in an ordinary browser, and for pure layout
work that is the quicker loop. It cannot answer anything that depends on
the native half: a Tauri command's actual return value, the window modes,
the store on disk, the gateway round-trip, or how WebKitGTK -- which is
the engine the device actually ships -- lays the page out. Chromium
agreeing with the design is not evidence that the device does.

This drives the built binary itself over WebDriver, which is Tauri's own
supported way in. The app runs unmodified: nothing is added to the
shipped binary to make this work, and there is no debug-only code path
holding the door open on a customer's device.

Needs two things installed once, neither of which ships:

    sudo apt install webkitgtk-webdriver     # the WebKitWebDriver binary
    cargo install tauri-driver               # Tauri's WebDriver shim

Then `make ui-drive` runs it, or import App from here for a scenario of
your own. Deliberately dependency-free -- WebDriver is JSON over HTTP,
and a debugging tool that needs its own package tree is one that stops
working the week you need it.
"""

import base64
import json
import os
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
APP = os.environ.get("AGENTIC_OS_APP", f"{REPO}/ui/src-tauri/target/release/matoakaui")
PORT = int(os.environ.get("AGENTIC_OS_DRIVER_PORT", "4445"))
BASE = f"http://127.0.0.1:{PORT}"

# The app writes its store under XDG_DATA_HOME. Pointed somewhere
# throwaway by default so a scripted run cannot leave anything in the
# real one -- an automated pass once wrote its own window geometry into
# a developer's settings and the app opened in the wrong place
# afterwards, which is a confusing thing to debug on top of whatever you
# were actually debugging.
SANDBOX = os.environ.get("AGENTIC_OS_DRIVE_HOME", "/tmp/agentic-os-drive")


def _request(method, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(
        BASE + path, data=data, method=method,
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            return json.loads(response.read() or b"{}")
    except urllib.error.HTTPError as error:
        detail = error.read().decode()[:500]
        raise RuntimeError(f"{method} {path} -> {error.code}: {detail}") from None


class App:
    """One run of the app, driven from outside.

    Used as a context manager so the window and both drivers are torn
    down even when a scenario fails -- a stranded app holding the
    gateway session makes the next run behave differently, which reads
    as a flaky bug rather than a leaked process.
    """

    def __init__(self, env=None, sandbox=True, log=None):
        self.env = {**os.environ, "DISPLAY": os.environ.get("DISPLAY", ":0")}
        if sandbox:
            os.makedirs(SANDBOX, exist_ok=True)
            self.env["XDG_DATA_HOME"] = SANDBOX
        self.env.update(env or {})
        self.driver = None
        self.session = None
        # tauri-driver's own chatter -- useful when the app never comes
        # up at all, and nearly empty otherwise.
        self.driver_log = log or os.path.join(SANDBOX, "driver.log")

    @property
    def store(self):
        """The settings file this run reads and writes."""
        return os.path.join(self.env["XDG_DATA_HOME"], "com.agenticos.shell", "settings.json")

    def seed(self, **values):
        """Write the store before launch.

        Setup is a conversation with a language model, so sitting
        through it is minutes per run and different every time. Writing
        the same file the app writes puts a scenario straight onto the
        surface it is about. Call before entering the context.
        """
        os.makedirs(os.path.dirname(self.store), exist_ok=True)
        with open(self.store, "w") as handle:
            json.dump(values, handle)

    def __enter__(self):
        os.makedirs(os.path.dirname(self.driver_log), exist_ok=True)
        self._log = open(self.driver_log, "w")
        self.driver = subprocess.Popen(
            ["tauri-driver", "--port", str(PORT),
             # Its own port and the native driver's must differ; sharing
             # one leaves tauri-driver listening and WebKitWebDriver dead,
             # which presents as a connection that opens and then hangs.
             "--native-port", str(PORT + 1),
             "--native-driver", "/usr/bin/WebKitWebDriver"],
            env=self.env, stdout=self._log, stderr=subprocess.STDOUT,
            preexec_fn=os.setsid,
        )
        for _ in range(50):
            try:
                urllib.request.urlopen(BASE + "/status", timeout=1)
                break
            except Exception:
                time.sleep(0.2)
        created = _request("POST", "/session", {
            "capabilities": {"alwaysMatch": {"tauri:options": {"application": APP}}}
        })
        value = created.get("value", created)
        self.session = value["sessionId"]
        self.watch_console()
        return self

    def __exit__(self, *_):
        for close in (lambda: _request("DELETE", f"/session/{self.session}"),
                      lambda: os.killpg(os.getpgid(self.driver.pid), signal.SIGTERM)):
            try:
                close()
            except Exception:
                pass
        subprocess.run(["pkill", "-f", APP], check=False)
        try:
            self._log.close()
        except Exception:
            pass

    # -- what a scenario actually uses --------------------------------

    @property
    def app_log(self):
        """The shell's own log file, written by its logging plugin."""
        return os.path.join(self.env["XDG_DATA_HOME"], "com.agenticos.shell", "logs", "ui.log")

    def watch_console(self):
        """Start keeping what the page prints.

        WebKitGTK does not put the page's console anywhere this can read,
        so it is captured in the page instead. Only what happens after
        this call is kept -- for something thrown during startup, read
        `app_log` and the screenshot rather than expecting it here.
        Installed automatically when the app starts.
        """
        self.js("""
            if (!window.__driverLog) {
              window.__driverLog = [];
              for (const level of ['log', 'info', 'warn', 'error']) {
                const original = console[level].bind(console);
                console[level] = (...parts) => {
                  window.__driverLog.push(level + ': ' + parts.map(String).join(' '));
                  original(...parts);
                };
              }
              addEventListener('error', (e) => window.__driverLog.push('error: ' + e.message));
              addEventListener('unhandledrejection',
                (e) => window.__driverLog.push('error: unhandled ' + e.reason));
            }
        """)

    def log(self, match=None):
        """Everything worth reading when something went wrong.

        Both halves together: what the Rust side wrote to its log file,
        and what the page printed. A failure usually shows in one of
        them, and which one is not obvious before you look.
        """
        lines = []
        for path, label in ((self.app_log, "app"), (self.driver_log, "driver")):
            try:
                with open(path) as handle:
                    lines += [f"[{label}] {line}" for line in handle.read().splitlines() if line]
            except FileNotFoundError:
                pass
        lines += [f"[page] {entry}" for entry in (self.js("return window.__driverLog || []") or [])]
        return [line for line in lines if match is None or match.lower() in line.lower()]

    def js(self, script, *args):
        """Run JavaScript in the page and return its value.

        This is the one that earns the setup: it reads what the native
        layer really returned, not what a fixture said it would.
        """
        return _request("POST", f"/session/{self.session}/execute/sync",
                        {"script": script, "args": list(args)}).get("value")

    def screenshot(self, path):
        data = _request("GET", f"/session/{self.session}/screenshot")["value"]
        with open(path, "wb") as handle:
            handle.write(base64.b64decode(data))
        return path

    def element(self, selector):
        found = _request("POST", f"/session/{self.session}/element",
                         {"using": "css selector", "value": selector})
        return list(found["value"].values())[0]

    def click(self, selector):
        _request("POST", f"/session/{self.session}/element/{self.element(selector)}/click", {})

    def text(self, selector="body"):
        return self.js("return document.querySelector(arguments[0])?.innerText || ''", selector)

    def type_into(self, selector, words):
        """Put text in a field the way a person would.

        Assigning `.value` is not enough: the field is bound, and a
        framework that never sees an input event still believes the box
        is empty when the form is submitted.
        """
        self.js("""
            const el = document.querySelector(arguments[0]);
            const proto = Object.getOwnPropertyDescriptor(el.constructor.prototype, 'value');
            proto.set.call(el, arguments[1]);
            el.dispatchEvent(new Event('input', { bubbles: true }));
        """, selector, words)

    def settle(self, script, seconds=120):
        """Wait for something the app decides, not for a fixed delay.

        A turn takes as long as the model takes, and a sleep long enough
        to be safe on a slow reply is one that wastes minutes on every
        fast one.
        """
        deadline = time.time() + seconds
        while time.time() < deadline:
            if self.js(f"return !!({script})"):
                return True
            time.sleep(0.5)
        return False


def main():
    """A look around: land on the shell, then report what is really there."""
    app = App()
    app.seed(language="en", agentName="Ada", persona="balanced",
             onboardingStarted=True, onboardingComplete=True)
    with app:
        app.settle("document.querySelector('.pane, .pill')", seconds=30)
        print(f"route            : {app.js('return location.pathname')}")
        # Worth printing every run: on a display with no settings daemon
        # this silently goes negative and takes the whole layout with it.
        print(f"devicePixelRatio : {app.js('return window.devicePixelRatio')}")
        print(f"window mode      : {app.js('return await __TAURI_INTERNALS__.invoke(\"window_mode_get\")')}")
        print(f"active chat      : {app.js('return await __TAURI_INTERNALS__.invoke(\"sessions_active\")')}")
        shot = os.environ.get("AGENTIC_OS_SHOT", "/tmp/agentic-os-drive/shell.png")
        os.makedirs(os.path.dirname(shot), exist_ok=True)
        print(f"screenshot       : {app.screenshot(shot)}")


if __name__ == "__main__":
    sys.exit(main())
