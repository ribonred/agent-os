// Recording what the owner says, for exactly as long as they hold the
// control down.
//
// This is the one part of the shell that opens a microphone, and the
// rules it follows are about trust rather than about audio. The stream
// is opened when the control goes down and released when it comes up --
// not held open between turns, so the recording light on a USB headset
// goes out when the owner lets go and the device is visibly not
// listening. A microphone that stays warm because it is cheaper than
// reopening it is exactly the behaviour that would make someone put this
// device in a drawer.
//
// The engine on the device is WebKitGTK, which records through
// GStreamer and does not offer the same containers a Chromium-based
// browser does. Nothing here names a format it expects to get: it asks
// what is supported, records in that, and reports what it actually
// produced so the transcription side can label the upload correctly.

/// Why a recording could not start, in terms the interface can say
/// something useful about. The engine's own message goes to the log.
export type CaptureFailure = "no-device" | "refused" | "unsupported" | "failed";

export class CaptureError extends Error {
  reason: CaptureFailure;

  constructor(reason: CaptureFailure, message: string) {
    super(message);
    this.name = "CaptureError";
    this.reason = reason;
  }
}

export type Recording = {
  bytes: Uint8Array;
  /** What the engine actually recorded, not what was asked for. */
  mime: string;
};

/// Preferred first, but every one of these is a guess about the engine.
/// Whatever it accepts is what gets recorded.
const CONTAINERS = [
  "audio/webm;codecs=opus",
  "audio/webm",
  "audio/ogg;codecs=opus",
  "audio/ogg",
  "audio/mp4",
  "audio/mpeg",
  "audio/wav",
];

function supportedContainer(): string | null {
  if (typeof MediaRecorder === "undefined") return null;
  for (const type of CONTAINERS) {
    if (MediaRecorder.isTypeSupported?.(type)) return type;
  }
  // An engine with a recorder but no opinion about types still records;
  // it just picks for itself, and it will tell us what it picked.
  return "";
}

function failureOf(error: unknown): CaptureFailure {
  const name = (error as { name?: string })?.name ?? "";
  // Nothing plugged in. The mini-PC the product ships on has no
  // microphone of its own, so this is the ordinary case on a unit whose
  // headset is not connected -- not a fault, and worth saying plainly.
  if (name === "NotFoundError" || name === "DevicesNotFoundError") return "no-device";
  // The engine or the desktop refused. On this device the shell grants
  // its own capture permission natively, so seeing this means something
  // outside the app is holding it.
  if (name === "NotAllowedError" || name === "SecurityError" || name === "PermissionDeniedError") {
    return "refused";
  }
  // In use by something else, or a device that disappeared mid-open.
  if (name === "NotReadableError" || name === "AbortError") return "failed";
  return "failed";
}

/// One recording, from the moment the control goes down.
///
/// Kept as a live object rather than a start/stop pair of functions
/// because a recording owns an operating-system resource: whoever starts
/// one is responsible for ending it, and `stop()` always releases the
/// microphone even when the recording itself failed.
export class VoiceRecording {
  private recorder: MediaRecorder;
  private stream: MediaStream;
  private parts: Blob[] = [];
  private ended: Promise<void>;

  private constructor(recorder: MediaRecorder, stream: MediaStream) {
    this.recorder = recorder;
    this.stream = stream;
    this.recorder.ondataavailable = (event) => {
      if (event.data && event.data.size > 0) this.parts.push(event.data);
    };
    this.ended = new Promise((resolve) => {
      this.recorder.onstop = () => resolve();
      this.recorder.onerror = () => resolve();
    });
  }

  static async start(): Promise<VoiceRecording> {
    if (!navigator.mediaDevices?.getUserMedia) {
      throw new CaptureError("unsupported", "this engine has no microphone support");
    }
    const container = supportedContainer();
    if (container === null) {
      throw new CaptureError("unsupported", "this engine cannot record audio");
    }

    let stream: MediaStream;
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        // A counter is a noisy room and the speaker is right beside the
        // microphone. Asked for rather than assumed: an engine that does
        // not have them ignores the hints instead of failing.
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        },
      });
    } catch (error) {
      console.error("could not open the microphone", error);
      throw new CaptureError(failureOf(error), String(error));
    }

    try {
      const recorder = container
        ? new MediaRecorder(stream, { mimeType: container })
        : new MediaRecorder(stream);
      const recording = new VoiceRecording(recorder, stream);
      recorder.start();
      return recording;
    } catch (error) {
      // The stream is open at this point and would otherwise stay open.
      for (const track of stream.getTracks()) track.stop();
      console.error("could not start recording", error);
      throw new CaptureError("unsupported", String(error));
    }
  }

  /// Ends the recording and releases the microphone. Safe to call twice:
  /// the control can come up more than once (a pointer that leaves the
  /// button and is then released), and neither should be an error.
  async stop(): Promise<Recording> {
    const mime = this.recorder.mimeType || "audio/webm";
    if (this.recorder.state !== "inactive") {
      this.recorder.stop();
      await this.ended;
    }
    for (const track of this.stream.getTracks()) track.stop();

    const blob = new Blob(this.parts, { type: mime });
    const bytes = new Uint8Array(await blob.arrayBuffer());
    return { bytes, mime };
  }

  /// Throws the recording away and releases the microphone without
  /// producing anything -- the owner changed their mind, or the turn was
  /// abandoned.
  discard() {
    if (this.recorder.state !== "inactive") {
      try {
        this.recorder.stop();
      } catch {
        // Already stopping. The tracks below are what actually matters.
      }
    }
    for (const track of this.stream.getTracks()) track.stop();
    this.parts = [];
  }
}
