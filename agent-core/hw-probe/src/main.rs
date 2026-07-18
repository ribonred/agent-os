// Thin CLI over the hw-probe library: run the probe once, print JSON.
// All detection/policy logic lives in lib.rs so any host process can
// link it directly -- one implementation, however many consumers.

fn main() {
    // No real credential store or vertical config is reachable from the
    // standalone CLI -- both false is honest here. In-process consumers
    // call hw_probe::probe() with the real values instead.
    let result = hw_probe::probe(false, false);
    let json = serde_json::to_string_pretty(&result).expect("ProbeResult is always serializable");
    println!("{json}");
}
