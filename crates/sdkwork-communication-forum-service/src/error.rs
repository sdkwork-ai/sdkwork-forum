use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForumServiceError {
    NotImplemented {
        operation: &'static str,
    },
    Validation {
        message: String,
    },
    PermissionDenied {
        permission: &'static str,
    },
    Conflict {
        message: String,
    },
    NotFound {
        resource: &'static str,
        id: Option<String>,
    },
    OptimisticLock {
        resource: &'static str,
        id: String,
    },
    Sanctioned {
        reason: String,
    },
    BoardClosed {
        board_id: i64,
    },
    TopicLocked {
        topic_id: i64,
    },
    DuplicateReport {
        target_type: String,
        target_id: i64,
    },
    IdempotencyConflict {
        key: String,
    },
    Internal {
        message: String,
    },
}

impl ForumServiceError {
    pub fn not_implemented(operation: &'static str) -> Self {
        Self::NotImplemented { operation }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn permission_denied(permission: &'static str) -> Self {
        Self::PermissionDenied { permission }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn not_found(resource: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            id: Some(id.into()),
        }
    }

    pub fn not_found_no_id(resource: &'static str) -> Self {
        Self::NotFound { resource, id: None }
    }

    pub fn optimistic_lock(resource: &'static str, id: impl Into<String>) -> Self {
        Self::OptimisticLock {
            resource,
            id: id.into(),
        }
    }

    pub fn sanctioned(reason: impl Into<String>) -> Self {
        Self::Sanctioned {
            reason: reason.into(),
        }
    }

    pub fn board_closed(board_id: i64) -> Self {
        Self::BoardClosed { board_id }
    }

    pub fn topic_locked(topic_id: i64) -> Self {
        Self::TopicLocked { topic_id }
    }

    pub fn duplicate_report(target_type: impl Into<String>, target_id: i64) -> Self {
        Self::DuplicateReport {
            target_type: target_type.into(),
            target_id,
        }
    }

    pub fn idempotency_conflict(key: impl Into<String>) -> Self {
        Self::IdempotencyConflict { key: key.into() }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotImplemented { .. } => "forum.not_implemented",
            Self::Validation { .. } => "forum.validation",
            Self::PermissionDenied { .. } => "forum.permission_denied",
            Self::Conflict { .. } => "forum.conflict",
            Self::NotFound { .. } => "forum.not_found",
            Self::OptimisticLock { .. } => "forum.optimistic_lock",
            Self::Sanctioned { .. } => "forum.sanctioned",
            Self::BoardClosed { .. } => "forum.board_closed",
            Self::TopicLocked { .. } => "forum.topic_locked",
            Self::DuplicateReport { .. } => "forum.duplicate_report",
            Self::IdempotencyConflict { .. } => "forum.idempotency_conflict",
            Self::Internal { .. } => "forum.internal",
        }
    }

    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::NotImplemented { .. } => 501,
            Self::Validation { .. } => 400,
            Self::PermissionDenied { .. } => 403,
            Self::Conflict { .. } => 409,
            Self::NotFound { .. } => 404,
            Self::OptimisticLock { .. } => 409,
            Self::Sanctioned { .. } => 403,
            Self::BoardClosed { .. } => 403,
            Self::TopicLocked { .. } => 403,
            Self::DuplicateReport { .. } => 409,
            Self::IdempotencyConflict { .. } => 409,
            Self::Internal { .. } => 500,
        }
    }
}

impl fmt::Display for ForumServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { operation } => write!(f, "not implemented: {operation}"),
            Self::Validation { message } => write!(f, "validation: {message}"),
            Self::PermissionDenied { permission } => write!(f, "permission denied: {permission}"),
            Self::Conflict { message } => write!(f, "conflict: {message}"),
            Self::NotFound { resource, id } => match id {
                Some(id) => write!(f, "{resource} not found: {id}"),
                None => write!(f, "{resource} not found"),
            },
            Self::OptimisticLock { resource, id } => {
                write!(f, "optimistic lock failed for {resource}: {id}")
            }
            Self::Sanctioned { reason } => write!(f, "user sanctioned: {reason}"),
            Self::BoardClosed { board_id } => write!(f, "board closed: {board_id}"),
            Self::TopicLocked { topic_id } => write!(f, "topic locked: {topic_id}"),
            Self::DuplicateReport {
                target_type,
                target_id,
            } => {
                write!(f, "duplicate report for {target_type}:{target_id}")
            }
            Self::IdempotencyConflict { key } => write!(f, "idempotency conflict: {key}"),
            Self::Internal { message } => write!(f, "internal error: {message}"),
        }
    }
}

impl std::error::Error for ForumServiceError {}
