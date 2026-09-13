use crate::models::{ApprovalRecord, ArtifactRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    MissingApproval,
    NotApproved,
    VersionMismatch,
    ModelOnSideEffect,
}

impl PolicyError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingApproval => "Maro blocked: explicit Sam approval is missing",
            Self::NotApproved => "Maro blocked: approval is not APPROVED",
            Self::VersionMismatch => "Maro blocked: approval is stale or version-mismatched",
            Self::ModelOnSideEffect => "Policy blocked: side-effect path must not invoke a model",
        }
    }
}

impl From<PolicyError> for String {
    fn from(value: PolicyError) -> Self {
        value.as_str().into()
    }
}

/// P1–P3: version-bound Sam approval is the only public-execute authorization.
pub fn assert_publish_allowed(
    approval: Option<&ApprovalRecord>,
    content: &ArtifactRecord,
    visual: &ArtifactRecord,
    package: &ArtifactRecord,
) -> Result<(), PolicyError> {
    let a = approval.ok_or(PolicyError::MissingApproval)?;
    if a.status != "APPROVED" {
        return Err(PolicyError::NotApproved);
    }
    if a.content_artifact_id != content.artifact_id
        || a.content_version != content.version
        || a.creative_artifact_id != visual.artifact_id
        || a.creative_version != visual.version
        || a.package_artifact_id != package.artifact_id
        || a.package_version != package.version
    {
        return Err(PolicyError::VersionMismatch);
    }
    Ok(())
}

pub fn revision_target(feedback: Option<&str>) -> &'static str {
    let text = feedback.unwrap_or("").to_lowercase();
    if text.is_empty() {
        return "production";
    }
    if ["strategy", "audience", "angle", "brief"].iter().any(|k| text.contains(k)) {
        return "adam";
    }
    if ["visual", "proof", "creative"].iter().any(|k| text.contains(k)) {
        return "jax";
    }
    "brain"
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_feedback_targets_production() {
        assert_eq!(revision_target(None), "production");
        assert_eq!(revision_target(Some("please change the angle")), "adam");
        assert_eq!(revision_target(Some("visual proof is weak")), "jax");
        assert_eq!(revision_target(Some("tighten the hook")), "brain");
    }
}
