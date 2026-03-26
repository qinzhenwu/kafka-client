use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroupInfo {
    pub group_id: String,
    pub state: String,
    pub members: Vec<ConsumerGroupMember>,
    pub partitions: Vec<GroupPartitionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerGroupMember {
    pub member_id: String,
    pub client_id: String,
    pub client_host: String,
    pub assignment: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPartitionInfo {
    pub topic: String,
    pub partition: i32,
    pub current_offset: i64,
    pub end_offset: i64,
    pub lag: i64,
    pub member_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagInfo {
    pub topic: String,
    pub partition: i32,
    pub current_offset: i64,
    pub end_offset: i64,
    pub lag: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetOffsetRequest {
    pub group_id: String,
    pub topic: String,
    pub partitions: Vec<i32>,
    pub offset: ResetOffsetType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ResetOffsetType {
    Earliest,
    Latest,
    Timestamp(i64),
    Specific(i64),
}