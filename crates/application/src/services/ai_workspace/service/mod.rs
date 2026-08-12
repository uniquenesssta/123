mod context;
mod operations;
mod sessions;

#[derive(Debug, Default)]
pub(crate) struct AiWorkspaceService;

impl AiWorkspaceService {
    pub(crate) fn new() -> Self {
        Self
    }
}
