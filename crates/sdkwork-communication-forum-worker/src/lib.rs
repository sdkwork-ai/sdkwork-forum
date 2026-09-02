use sdkwork_communication_forum_service::domain::commands::{
    FanoutNotificationsCommand, ListModerationQueueCommand, PublishOutboxCommand,
    RebuildSearchProjectionCommand, RebuildStatsCommand,
};
use sdkwork_communication_forum_service::ports::repository::ForumRepository;
use sdkwork_communication_forum_service::ForumService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForumWorkerJob {
    PublishOutbox,
    RebuildSearchProjection,
    RebuildStats,
    EvaluateModerationPolicy,
    FanoutNotifications,
}

pub struct ForumWorker<R: ForumRepository> {
    service: ForumService<R>,
}

impl<R: ForumRepository> ForumWorker<R> {
    pub fn new(service: ForumService<R>) -> Self {
        Self { service }
    }

    pub fn run_job(
        &self,
        job: ForumWorkerJob,
        ctx: &sdkwork_communication_forum_service::value_objects::ForumRequestContext,
    ) -> Result<(), String> {
        match job {
            ForumWorkerJob::PublishOutbox => self.publish_outbox(ctx),
            ForumWorkerJob::RebuildSearchProjection => self.rebuild_search_projection(ctx),
            ForumWorkerJob::RebuildStats => self.rebuild_stats(ctx),
            ForumWorkerJob::EvaluateModerationPolicy => self.evaluate_moderation_policy(ctx),
            ForumWorkerJob::FanoutNotifications => self.fanout_notifications(ctx),
        }
    }

    fn publish_outbox(
        &self,
        ctx: &sdkwork_communication_forum_service::value_objects::ForumRequestContext,
    ) -> Result<(), String> {
        self.service
            .publish_pending_outbox(ctx, PublishOutboxCommand { limit: 100 })
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn rebuild_search_projection(
        &self,
        ctx: &sdkwork_communication_forum_service::value_objects::ForumRequestContext,
    ) -> Result<(), String> {
        self.service
            .rebuild_search_projection(
                ctx,
                RebuildSearchProjectionCommand {
                    scope: None,
                    board_id: None,
                },
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn rebuild_stats(
        &self,
        ctx: &sdkwork_communication_forum_service::value_objects::ForumRequestContext,
    ) -> Result<(), String> {
        self.service
            .rebuild_stats(
                ctx,
                RebuildStatsCommand {
                    scope: Some("all".to_string()),
                },
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn evaluate_moderation_policy(
        &self,
        ctx: &sdkwork_communication_forum_service::value_objects::ForumRequestContext,
    ) -> Result<(), String> {
        self.service
            .list_moderation_queue(
                ctx,
                ListModerationQueueCommand {
                    status_filter: Some("pending".to_string()),
                    severity_filter: None,
                    cursor: None,
                    limit: 100,
                },
            )
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn fanout_notifications(
        &self,
        ctx: &sdkwork_communication_forum_service::value_objects::ForumRequestContext,
    ) -> Result<(), String> {
        self.service
            .fanout_notifications(ctx, FanoutNotificationsCommand { limit: 100 })
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}
