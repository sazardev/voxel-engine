use platform::Capabilities;

const LOW_GPU_MEMORY_MB: u32 = 2048;
const MID_GPU_MEMORY_MB: u32 = 4096;
const ULTRA_GPU_MEMORY_MB: u32 = 8192;

/// Visual quality level. Tiers add or remove passes and resolution; art direction stays the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    Low,
    Mid,
    High,
    Ultra,
}

impl Tier {
    /// Frame time this tier must stay under, in milliseconds (see `docs/stack/quality-gates.md`).
    #[must_use]
    pub const fn frame_budget_ms(self) -> f32 {
        match self {
            Self::Low => 33.3,
            Self::Mid | Self::High | Self::Ultra => 16.7,
        }
    }
}

/// Picks the tier from probed capabilities; the player's choice wins when present.
/// Touch devices never default above `Mid` (battery and heat).
#[must_use]
pub fn select_tier(capabilities: &Capabilities, player_override: Option<Tier>) -> Tier {
    if let Some(tier) = player_override {
        return tier;
    }
    let by_hardware = tier_for_hardware(capabilities);
    if capabilities.has_touch {
        by_hardware.min(Tier::Mid)
    } else {
        by_hardware
    }
}

fn tier_for_hardware(capabilities: &Capabilities) -> Tier {
    if !capabilities.supports_compute || capabilities.gpu_memory_mb < LOW_GPU_MEMORY_MB {
        Tier::Low
    } else if capabilities.gpu_memory_mb < MID_GPU_MEMORY_MB {
        Tier::Mid
    } else if capabilities.gpu_memory_mb < ULTRA_GPU_MEMORY_MB {
        Tier::High
    } else {
        Tier::Ultra
    }
}

#[cfg(test)]
mod tests {
    use platform::GpuApi;

    use super::*;

    fn desktop(gpu_memory_mb: u32) -> Capabilities {
        Capabilities {
            gpu_api: GpuApi::Vulkan,
            gpu_memory_mb,
            system_memory_mb: 16_384,
            max_texture_size: 16_384,
            supports_compute: true,
            has_touch: false,
            has_threads: true,
        }
    }

    #[test]
    fn the_default_capabilities_get_the_low_tier() {
        assert_eq!(select_tier(&Capabilities::default(), None), Tier::Low);
    }

    #[test]
    fn a_big_desktop_gpu_gets_ultra() {
        assert_eq!(select_tier(&desktop(12_288), None), Tier::Ultra);
    }

    #[test]
    fn a_touch_device_never_defaults_above_mid() {
        let phone = Capabilities {
            has_touch: true,
            ..desktop(12_288)
        };

        assert_eq!(select_tier(&phone, None), Tier::Mid);
    }

    #[test]
    fn the_player_override_wins_over_hardware() {
        assert_eq!(select_tier(&desktop(12_288), Some(Tier::Low)), Tier::Low);
    }

    #[test]
    fn only_the_low_tier_has_the_thirty_fps_budget() {
        assert!(Tier::Low.frame_budget_ms() > Tier::Mid.frame_budget_ms());
        assert!((Tier::High.frame_budget_ms() - Tier::Mid.frame_budget_ms()).abs() < f32::EPSILON);
    }
}
