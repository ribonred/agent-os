// Thin CLI over the hw-probe library: run the probe once, print JSON.
// All detection/policy logic lives in lib.rs so the orchestrator links it
// directly -- one implementation, two consumers.

fn main() {
    // No real credential store or vertical config is reachable from the
    // standalone CLI -- both false is honest here. The orchestrator calls
    // hw_probe::probe() with the real values instead.
    let result = hw_probe::probe(false, false);
    let json = serde_json::to_string_pretty(&result).expect("ProbeResult is always serializable");
    println!("{json}");
}
