use crate::domain::commands::*;
use crate::domain::models::*;
use crate::domain::results::*;
use crate::error::ForumServiceError;
use crate::value_objects::ForumRequestContext;

pub trait ForumRepository {
    fn list_node_tree(
        &self,
        ctx: &ForumRequestContext,
        command: &ListNodeTreeCommand,
    ) -> Result<NodeTreeResult, ForumServiceError>;
    fn list_nodes(
        &self,
        ctx: &ForumRequestContext,
        command: &ListNodesCommand,
    ) -> Result<NodePageResult, ForumServiceError>;
    fn list_topics(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicsCommand,
    ) -> Result<TopicPageResult, ForumServiceError>;
    fn retrieve_topic_by_slug(
        &self,
        ctx: &ForumRequestContext,
        command: &RetrieveTopicBySlugCommand,
    ) -> Result<ForumTopic, ForumServiceError>;
    fn create_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateTopicCommand,
    ) -> Result<ForumTopic, ForumServiceError>;
    fn retrieve_topic(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<ForumTopic, ForumServiceError>;
    fn update_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateTopicCommand,
    ) -> Result<ForumTopic, ForumServiceError>;
    fn delete_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &DeleteTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn list_replies(
        &self,
        ctx: &ForumRequestContext,
        command: &ListRepliesCommand,
    ) -> Result<ReplyPageResult, ForumServiceError>;
    fn create_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReplyCommand,
    ) -> Result<ForumReply, ForumServiceError>;
    fn update_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateReplyCommand,
    ) -> Result<ForumReply, ForumServiceError>;
    fn delete_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &DeleteReplyCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn accept_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &AcceptReplyCommand,
    ) -> Result<ForumTopic, ForumServiceError>;
    fn clear_accepted_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &ClearAcceptedReplyCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn create_report(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReportCommand,
    ) -> Result<CommandResult, ForumServiceError>;

    fn query_search(
        &self,
        ctx: &ForumRequestContext,
        command: &QuerySearchCommand,
    ) -> Result<SearchResult, ForumServiceError>;
    fn list_moderation_queue(
        &self,
        ctx: &ForumRequestContext,
        command: &ListModerationQueueCommand,
    ) -> Result<ModerationQueueResult, ForumServiceError>;
    fn create_moderation_decision(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateModerationDecisionCommand,
    ) -> Result<ModerationDecisionResult, ForumServiceError>;
    fn rebuild_search_projection(
        &self,
        ctx: &ForumRequestContext,
        command: &RebuildSearchProjectionCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn rebuild_stats(
        &self,
        ctx: &ForumRequestContext,
        command: &RebuildStatsCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn list_pending_outbox_events(
        &self,
        ctx: &ForumRequestContext,
        command: &PublishOutboxCommand,
    ) -> Result<Vec<ForumOutboxEvent>, ForumServiceError>;
    fn mark_outbox_published(
        &self,
        ctx: &ForumRequestContext,
        event_id: i64,
    ) -> Result<(), ForumServiceError>;
    fn list_topic_revisions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicRevisionsCommand,
    ) -> Result<TopicRevisionPageResult, ForumServiceError>;
    fn list_reply_revisions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListReplyRevisionsCommand,
    ) -> Result<ReplyRevisionPageResult, ForumServiceError>;
    fn create_poll_vote(
        &self,
        ctx: &ForumRequestContext,
        command: &CreatePollVoteCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn create_reaction(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReactionCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn create_vote(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateVoteCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn update_bookmark(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateBookmarkCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn update_read_state(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateReadStateCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn pin_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &PinTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn feature_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &FeatureTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn lock_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &LockTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn unpin_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &PinTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn unfeature_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &FeatureTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn unlock_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &LockTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn move_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &MoveTopicCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn create_node(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateNodeCommand,
    ) -> Result<ForumNode, ForumServiceError>;
    fn update_node(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateNodeCommand,
    ) -> Result<ForumNode, ForumServiceError>;
    fn delete_node(
        &self,
        ctx: &ForumRequestContext,
        command: &DeleteNodeCommand,
    ) -> Result<CommandResult, ForumServiceError>;
    fn list_moderation_cases(
        &self,
        ctx: &ForumRequestContext,
        command: &ListModerationCasesCommand,
    ) -> Result<ModerationCasePageResult, ForumServiceError>;
    fn create_moderation_case(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateModerationCaseCommand,
    ) -> Result<ForumModerationCase, ForumServiceError>;
    fn retrieve_moderation_case(
        &self,
        ctx: &ForumRequestContext,
        command: &RetrieveModerationCaseCommand,
    ) -> Result<ForumModerationCase, ForumServiceError>;
    fn list_sanctions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListSanctionsCommand,
    ) -> Result<SanctionPageResult, ForumServiceError>;
    fn create_sanction(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateSanctionCommand,
    ) -> Result<ForumSanction, ForumServiceError>;
    fn update_sanction(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateSanctionCommand,
    ) -> Result<ForumSanction, ForumServiceError>;
    fn list_reputation_rules(
        &self,
        ctx: &ForumRequestContext,
        command: &ListReputationRulesCommand,
    ) -> Result<ReputationRulePageResult, ForumServiceError>;
    fn create_reputation_rule(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReputationRuleCommand,
    ) -> Result<ForumReputationRule, ForumServiceError>;
    fn list_reputation_ledger(
        &self,
        ctx: &ForumRequestContext,
        command: &ListReputationLedgerCommand,
    ) -> Result<ReputationLedgerPageResult, ForumServiceError>;
    fn list_trust_levels(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTrustLevelsCommand,
    ) -> Result<TrustLevelPageResult, ForumServiceError>;
    fn create_trust_level(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateTrustLevelCommand,
    ) -> Result<ForumTrustLevel, ForumServiceError>;
    fn list_badges(
        &self,
        ctx: &ForumRequestContext,
        command: &ListBadgesCommand,
    ) -> Result<BadgePageResult, ForumServiceError>;
    fn create_badge(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateBadgeCommand,
    ) -> Result<ForumBadge, ForumServiceError>;
    fn list_board_stats(
        &self,
        ctx: &ForumRequestContext,
        command: &ListBoardStatsCommand,
    ) -> Result<BoardStatsPageResult, ForumServiceError>;
    fn list_topic_stats(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicStatsCommand,
    ) -> Result<TopicStatsPageResult, ForumServiceError>;
    fn create_audit_action(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateAuditActionCommand,
    ) -> Result<ForumAuditAction, ForumServiceError>;
    fn list_audit_actions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListAuditActionsCommand,
    ) -> Result<AuditActionPageResult, ForumServiceError>;
    fn list_tags(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTagsCommand,
    ) -> Result<TagPageResult, ForumServiceError>;
    fn list_topic_prefixes(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicPrefixesCommand,
    ) -> Result<TopicPrefixPageResult, ForumServiceError>;
    fn create_topic_prefix(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateTopicPrefixCommand,
    ) -> Result<ForumTopicPrefix, ForumServiceError>;
    fn create_space(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateSpaceCommand,
    ) -> Result<ForumSpace, ForumServiceError>;
    fn update_space(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateSpaceCommand,
    ) -> Result<ForumSpace, ForumServiceError>;
    fn create_attachment(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateAttachmentCommand,
    ) -> Result<ForumAttachment, ForumServiceError>;
    fn create_subscription(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateSubscriptionCommand,
    ) -> Result<ForumSubscription, ForumServiceError>;
    fn update_subscription(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateSubscriptionCommand,
    ) -> Result<ForumSubscription, ForumServiceError>;
    fn list_subscriptions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListSubscriptionsCommand,
    ) -> Result<SubscriptionPageResult, ForumServiceError>;
    fn check_space_has_topics(
        &self,
        ctx: &ForumRequestContext,
        space_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn check_node_cycle(
        &self,
        ctx: &ForumRequestContext,
        node_id: i64,
        new_parent_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn check_node_is_board(
        &self,
        ctx: &ForumRequestContext,
        node_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn check_board_exists(
        &self,
        ctx: &ForumRequestContext,
        board_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn check_owner_exists(
        &self,
        ctx: &ForumRequestContext,
        owner_type: &str,
        owner_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn check_poll_exists(
        &self,
        ctx: &ForumRequestContext,
        poll_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn count_poll_votes(
        &self,
        ctx: &ForumRequestContext,
        poll_id: i64,
    ) -> Result<i64, ForumServiceError>;
    fn check_poll_selection_mode(
        &self,
        ctx: &ForumRequestContext,
        poll_id: i64,
    ) -> Result<String, ForumServiceError>;
    fn check_active_vote(
        &self,
        ctx: &ForumRequestContext,
        target_type: &str,
        target_id: i64,
        actor_user_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn check_active_sanctions(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<Vec<ForumSanction>, ForumServiceError>;
    fn check_active_appeal(
        &self,
        ctx: &ForumRequestContext,
        sanction_id: Option<i64>,
        case_id: Option<i64>,
        appellant_user_id: i64,
    ) -> Result<bool, ForumServiceError>;
    fn count_topics_in_space(
        &self,
        ctx: &ForumRequestContext,
        space_id: i64,
    ) -> Result<i64, ForumServiceError>;
    fn get_next_revision_no(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<i32, ForumServiceError>;
    fn get_next_reply_no(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<i32, ForumServiceError>;
    fn get_next_case_no(
        &self,
        ctx: &ForumRequestContext,
        tenant_id: i64,
    ) -> Result<String, ForumServiceError>;
    fn check_duplicate_queue_item(
        &self,
        ctx: &ForumRequestContext,
        target_type: &str,
        target_id: i64,
        source_type: &str,
    ) -> Result<bool, ForumServiceError>;
    fn check_idempotency_key(
        &self,
        ctx: &ForumRequestContext,
        key: &str,
        operation_id: &str,
    ) -> Result<Option<ForumIdempotencyRecord>, ForumServiceError>;
    fn check_message_id_exists(
        &self,
        ctx: &ForumRequestContext,
        source_system: &str,
        message_id: &str,
        consumer_name: &str,
    ) -> Result<bool, ForumServiceError>;
    fn check_message_payload_hash(
        &self,
        ctx: &ForumRequestContext,
        source_system: &str,
        message_id: &str,
        consumer_name: &str,
        payload_hash: &str,
    ) -> Result<bool, ForumServiceError>;
    fn get_reputation_balance(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<i64, ForumServiceError>;
    fn get_topic_stats(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<ForumTopicStats, ForumServiceError>;
    fn get_board_stats(
        &self,
        ctx: &ForumRequestContext,
        board_id: i64,
    ) -> Result<ForumBoardStats, ForumServiceError>;
    fn get_member_stats(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<ForumMemberStats, ForumServiceError>;
    fn update_tag_usage_count(
        &self,
        ctx: &ForumRequestContext,
        tag_id: i64,
    ) -> Result<(), ForumServiceError>;
    fn update_unread_count(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
        user_id: i64,
    ) -> Result<(), ForumServiceError>;
    fn get_notification_preferences(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
        event_type: &str,
    ) -> Result<Vec<ForumNotificationPreference>, ForumServiceError>;
    fn insert_outbox_event(
        &self,
        ctx: &ForumRequestContext,
        event: &ForumOutboxEvent,
    ) -> Result<(), ForumServiceError>;
    fn update_topic_stats(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<(), ForumServiceError>;
    fn update_board_stats(
        &self,
        ctx: &ForumRequestContext,
        board_id: i64,
    ) -> Result<(), ForumServiceError>;
    fn update_member_stats(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<(), ForumServiceError>;
}
