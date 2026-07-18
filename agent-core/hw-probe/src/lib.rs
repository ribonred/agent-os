//! Hardware capability detection and the local/cloud routing default.
//!
//! Library-first on purpose: agent-side consumers link this crate and
//! call the probe directly, so there is exactly one implementation of the
//! tier policy and routing lean. The `hw-probe` binary is a thin JSON printer
//! over the same functions for humans and scripts.

use std::fs;
use std::net::TcpStream;
use std::time::Duration;
use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuVendor {
    Intel,
    Nvidia,
    Amd,
    Unknown(String),
}

pub fn classify_vendor(vendor_id: &str) -> GpuVendor {
    match vendor_id.trim().to_lowercase().as_str() {
        "0x8086" => GpuVendor::Intel,
        "0x10de" => GpuVendor::Nvidia,
        "0x1002" => GpuVendor::Amd,
        other => GpuVendor::Unknown(other.to_string()),
    }
}

// Reads /sys/class/drm/card*/device/vendor -- the standard Linux kernel
// sysfs interface for PCI display devices. NOT validated against real
// hardware: this was written and tested in a WSL2 sandbox, which does not
// expose a real DRM/PCI topology.
//
// Confirmed concretely, not just assumed: this sandbox has a real NVIDIA
// RTX 3060 (nvidia-smi sees it fine, driver 595.79) that this function
// still finds nothing for, because WSL2 exposes GPU compute through its
// own paravirtualized path (/dev/dxg + userspace libs under
// /usr/lib/wsl/lib/), not the standard Linux DRM/PCI sysfs this function
// reads. /sys/class/drm here has no card* entries at all, only a
// "version" file, even with the GPU actively in use.
//
// This is not a bug to fix -- the real deployment target (NUC, DGX Spark)
// runs bare-metal NixOS, where /sys/class/drm is the correct mechanism.
// Adding a WSL-specific fallback would only serve this sandbox. Still
// must be checked against the actual NUC before this result is trusted
// for a routing decision.
fn detect_gpu_vendors() -> Vec<GpuVendor> {
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // "card0" is a GPU device; "card0-DP-1" is a connector on it --
            // skip connectors, they'd double-count the same physical GPU.
            if !name.starts_with("card") || name.contains('-') {
                return None;
            }
            fs::read_to_string(entry.path().join("device/vendor"))
                .ok()
                .map(|contents| classify_vendor(&contents))
        })
        .collect()
}

// Reads /sys/class/accel -- the modern Linux kernel interface for AI
// accelerators/NPUs. Same caveat as detect_gpu_vendors: unvalidated
// against real hardware, confirmed absent entirely on this sandbox.
fn has_npu() -> bool {
    fs::read_dir("/sys/class/accel")
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Low,
    Mid,
    High,
}

#[derive(Debug, Clone, Serialize)]
pub struct HwProfile {
    pub logical_cores: usize,
    pub total_memory_gib: f64,
    pub gpu_vendors: Vec<GpuVendor>,
    pub npu_present: bool,
}

/// Reads the hardware once. Callers should cache the result for the
/// process lifetime -- hardware doesn't change under a running system,
/// and re-probing per request would be pure waste.
pub fn probe_hardware() -> HwProfile {
    let mut sys = System::new_all();
    sys.refresh_all();

    HwProfile {
        logical_cores: sys.cpus().len(),
        total_memory_gib: sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
        gpu_vendors: detect_gpu_vendors(),
        npu_present: has_npu(),
    }
}

// Mirrors the tier policy in tasks/008: Intel integrated graphics alone
// does not count as an accelerator (that's exactly the NUC11PAHi3/Iris Xe
// case, which the policy explicitly puts in the low tier). NVIDIA/AMD
// entries are treated as accelerators.
//
// Known limitation, not solved here because neither device we're actually
// targeting hits it: this can't distinguish an integrated AMD APU's
// graphics from a discrete AMD GPU (both report vendor 0x1002), and it
// would misclassify an Intel discrete Arc GPU as non-accelerated (same
// vendor ID as Intel's integrated Xe). Fine for now -- our two known
// targets are Intel-integrated-only (NUC) and NVIDIA-discrete (DGX
// Spark) -- but revisit if AMD or Intel-discrete hardware enters scope.
pub fn classify_tier(profile: &HwProfile) -> Tier {
    let has_accelerator = profile.npu_present
        || profile
            .gpu_vendors
            .iter()
            .any(|v| matches!(v, GpuVendor::Nvidia | GpuVendor::Amd));

    if !has_accelerator {
        Tier::Low
    } else if profile.total_memory_gib >= 96.0 {
        // 96GiB floor, not 128 -- OS/reserved memory means a 128GB unified
        // memory system won't report the full 128 as available.
        Tier::High
    } else {
        Tier::Mid
    }
}

// Deliberately a raw TCP connect, not an HTTP client -- checking basic
// reachability doesn't need the dependency weight of reqwest/hyper.
// 1.1.1.1:443 is Cloudflare's anycast resolver: a stable IP with high
// uptime, and skipping DNS removes a failure variable from a check whose
// only job is "is there a network at all." A short timeout so a dead
// network doesn't stall boot -- this must never hang.
pub fn is_online() -> bool {
    "1.1.1.1:443"
        .parse()
        .map(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok())
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingLean {
    Local,
    Cloud,
}

// This is a *default lean*, not the full per-request routing decision --
// that needs conversation context and task-complexity assessment (e.g.
// low tier still runs small local tasks, only heavier requests go to
// cloud), which belongs to the agent runtime, not to a hardware
// probe. What this answers is narrower: absent any other signal, should
// this device default toward local or cloud.
//
// has_cloud_credentials matters independently of `online`: a reachable
// network is necessary but not sufficient to actually use the cloud tier
// -- an OpenRouter API key also has to be configured. Without one, "lean
// cloud" would just fail at the point of calling it, silently or
// otherwise. Caught this before it became a real bug, not after.
//
// vertical_forces_offline is a placeholder parameter -- no real vertical
// config exists yet to source it from. Wired here so the function has the
// right shape when that config does exist, not because it does anything
// today beyond what's tested.
pub fn decide_default_routing(
    tier: &Tier,
    online: bool,
    has_cloud_credentials: bool,
    vertical_forces_offline: bool,
) -> RoutingLean {
    if vertical_forces_offline || !online || !has_cloud_credentials {
        return RoutingLean::Local;
    }
    match tier {
        Tier::Low => RoutingLean::Cloud,
        Tier::Mid | Tier::High => RoutingLean::Local,
    }
}

// The full probe output, machine-readable. The hw-probe binary prints
// this as JSON; in-process consumers build it with real
// credential/vertical inputs.
#[derive(Debug, Clone, Serialize)]
pub struct ProbeResult {
    pub logical_cores: usize,
    pub total_memory_gib: f64,
    pub gpu_vendors: Vec<GpuVendor>,
    pub npu_present: bool,
    pub tier: Tier,
    pub online: bool,
    pub default_routing: RoutingLean,
}

/// One full probe pass: hardware + connectivity + routing lean.
/// `has_cloud_credentials` and `vertical_forces_offline` are inputs, not
/// probed -- they come from whatever credential store / vertical config
/// the caller actually has.
pub fn probe(has_cloud_credentials: bool, vertical_forces_offline: bool) -> ProbeResult {
    let profile = probe_hardware();
    let tier = classify_tier(&profile);
    let online = is_online();
    let default_routing =
        decide_default_routing(&tier, online, has_cloud_credentials, vertical_forces_offline);

    ProbeResult {
        logical_cores: profile.logical_cores,
        total_memory_gib: profile.total_memory_gib,
        gpu_vendors: profile.gpu_vendors,
        npu_present: profile.npu_present,
        tier,
        online,
        default_routing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(
        total_memory_gib: f64,
        gpu_vendors: Vec<GpuVendor>,
        npu_present: bool,
    ) -> HwProfile {
        HwProfile {
            logical_cores: 8,
            total_memory_gib,
            gpu_vendors,
            npu_present,
        }
    }

    #[test]
    fn classifies_known_vendor_ids() {
        assert_eq!(classify_vendor("0x8086"), GpuVendor::Intel);
        assert_eq!(classify_vendor("0x10de"), GpuVendor::Nvidia);
        assert_eq!(classify_vendor("0x1002"), GpuVendor::Amd);
    }

    #[test]
    fn classifies_unknown_vendor_id() {
        assert_eq!(
            classify_vendor("0x1234"),
            GpuVendor::Unknown("0x1234".to_string())
        );
    }

    #[test]
    fn handles_whitespace_and_case_in_vendor_id() {
        assert_eq!(classify_vendor("0x8086\n"), GpuVendor::Intel);
        assert_eq!(classify_vendor("0X8086"), GpuVendor::Intel);
    }

    #[test]
    fn intel_integrated_only_is_low_tier() {
        // The actual NUC11PAHi3 profile: Iris Xe, no NPU.
        assert_eq!(
            classify_tier(&profile(32.0, vec![GpuVendor::Intel], false)),
            Tier::Low
        );
    }

    #[test]
    fn no_gpu_at_all_is_low_tier() {
        assert_eq!(classify_tier(&profile(16.0, vec![], false)), Tier::Low);
    }

    #[test]
    fn discrete_gpu_modest_memory_is_mid_tier() {
        assert_eq!(
            classify_tier(&profile(32.0, vec![GpuVendor::Nvidia], false)),
            Tier::Mid
        );
    }

    #[test]
    fn npu_alone_is_mid_tier() {
        assert_eq!(classify_tier(&profile(16.0, vec![], true)), Tier::Mid);
    }

    #[test]
    fn dgx_spark_like_profile_is_high_tier() {
        // Fabricated: 128GB unified memory, Blackwell (NVIDIA). We don't
        // have this hardware to test against -- this documents the
        // expected classification so it's checked the moment real
        // DGX Spark numbers are available, same as the placeholder
        // hardware-configuration.nix pattern.
        assert_eq!(
            classify_tier(&profile(120.0, vec![GpuVendor::Nvidia], false)),
            Tier::High
        );
    }

    #[test]
    fn high_memory_without_accelerator_stays_low_tier() {
        // Memory alone never promotes a tier -- a beefy RAM box with no
        // GPU/NPU is still low-tier per the policy.
        assert_eq!(classify_tier(&profile(128.0, vec![], false)), Tier::Low);
    }

    #[test]
    fn is_online_reflects_real_network_state() {
        // Live check, not fabricated -- this sandbox has real network
        // access, so this validates actual behavior rather than just
        // logic. If this box goes offline the assertion (correctly)
        // fails; that's the point, not a flaky test to relax.
        assert!(is_online(), "expected this sandbox to have network access");
    }

    #[test]
    fn low_tier_online_with_credentials_defaults_to_cloud() {
        // The exact case from task 008's acceptance criteria: NUC-class
        // hardware, online, credentials configured, no override -- should
        // lean cloud without being told to.
        assert_eq!(
            decide_default_routing(&Tier::Low, true, true, false),
            RoutingLean::Cloud
        );
    }

    #[test]
    fn low_tier_offline_falls_back_to_local() {
        // No cloud to reach even if the tier would normally prefer it.
        assert_eq!(
            decide_default_routing(&Tier::Low, false, true, false),
            RoutingLean::Local
        );
    }

    #[test]
    fn low_tier_online_without_credentials_falls_back_to_local() {
        // The actual bug this test exists to prevent: a reachable network
        // is not the same as a usable cloud tier. No API key configured
        // means cloud would just fail if attempted -- must not lean cloud.
        assert_eq!(
            decide_default_routing(&Tier::Low, true, false, false),
            RoutingLean::Local
        );
    }

    #[test]
    fn mid_and_high_tier_default_local_even_when_online_with_credentials() {
        assert_eq!(
            decide_default_routing(&Tier::Mid, true, true, false),
            RoutingLean::Local
        );
        assert_eq!(
            decide_default_routing(&Tier::High, true, true, false),
            RoutingLean::Local
        );
    }

    #[test]
    fn vertical_override_forces_local_regardless_of_tier_connectivity_or_credentials() {
        // A money/health vertical saying "offline only" wins even for a
        // high-tier, online, credentialed device that would otherwise be
        // fine using cloud.
        assert_eq!(
            decide_default_routing(&Tier::High, true, true, true),
            RoutingLean::Local
        );
    }
}
