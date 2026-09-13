/// Capability registry for the kernel slice.
/// Roles own capabilities; implementations are deterministic, model, adapter, or policy.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionKind {
    ModelRun,
    CapabilityRun,
    AdapterRun,
    PolicyCheck,
}

impl ExecutionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ModelRun => "MODEL_RUN",
            Self::CapabilityRun => "CAPABILITY_RUN",
            Self::AdapterRun => "ADAPTER_RUN",
            Self::PolicyCheck => "POLICY_CHECK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability {
    pub id: &'static str,
    pub owner_role: &'static str,
    pub kind: ExecutionKind,
    pub may_use_model: bool,
}

pub const PACKAGE_OPPORTUNITY: Capability = Capability {
    id: "package_opportunity",
    owner_role: "saly",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const PRIORITIZE_AND_ASSIGN: Capability = Capability {
    id: "prioritize_and_assign",
    owner_role: "travis",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const COMPILE_BRIEF: Capability = Capability {
    id: "compile_brief",
    owner_role: "adam",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const WRITE_PUBLIC_COPY: Capability = Capability {
    id: "write_public_copy",
    owner_role: "brain",
    kind: ExecutionKind::ModelRun,
    may_use_model: true,
};
pub const DECIDE_CREATIVE: Capability = Capability {
    id: "decide_creative",
    owner_role: "jax",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const ASSEMBLE_PACKAGE: Capability = Capability {
    id: "assemble_package",
    owner_role: "system",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const RECORD_APPROVAL: Capability = Capability {
    id: "record_approval",
    owner_role: "sam",
    kind: ExecutionKind::PolicyCheck,
    may_use_model: false,
};
pub const VALIDATE_AND_PUBLISH: Capability = Capability {
    id: "validate_and_publish",
    owner_role: "maro",
    kind: ExecutionKind::AdapterRun,
    may_use_model: false,
};
pub const CAPTURE_SNAPSHOT: Capability = Capability {
    id: "capture_snapshot",
    owner_role: "system",
    kind: ExecutionKind::AdapterRun,
    may_use_model: false,
};
pub const INTERPRET_PERFORMANCE: Capability = Capability {
    id: "interpret_performance",
    owner_role: "lara",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const DECIDE_NEXT_CYCLE: Capability = Capability {
    id: "decide_next_cycle",
    owner_role: "travis",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};
pub const COMPILE_REVISION: Capability = Capability {
    id: "compile_revision",
    owner_role: "system",
    kind: ExecutionKind::CapabilityRun,
    may_use_model: false,
};

pub const INTENDED_PLATFORM: &str = "LINKEDIN";
pub const EXECUTION_TARGET: &str = "LOCAL_LEDGER";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maro_never_uses_a_model() {
        assert!(!VALIDATE_AND_PUBLISH.may_use_model);
        assert_eq!(VALIDATE_AND_PUBLISH.kind, ExecutionKind::AdapterRun);
    }
    #[test]
    fn only_brain_may_use_model_in_slice() {
        for cap in [
            PACKAGE_OPPORTUNITY,
            PRIORITIZE_AND_ASSIGN,
            COMPILE_BRIEF,
            DECIDE_CREATIVE,
            ASSEMBLE_PACKAGE,
            INTERPRET_PERFORMANCE,
            DECIDE_NEXT_CYCLE,
        ] {
            assert!(!cap.may_use_model, "{}", cap.id);
        }
        assert!(WRITE_PUBLIC_COPY.may_use_model);
    }
}
