use crate::error::{KafkaClientError, Result};
use crate::models::{ClusterConfig, ConsumerGroupInfo, ConsumerGroupMember, GroupPartitionInfo, LagInfo};
use rdkafka::consumer::BaseConsumer;
use rdkafka::consumer::Consumer;
use std::sync::Arc;

pub struct KafkaConsumerGroupOps<'a> {
    admin: &'a rdkafka::admin::AdminClient<rdkafka::client::DefaultClientContext>,
    consumer: &'a Arc<BaseConsumer>,
    config: &'a ClusterConfig,
}

impl<'a> KafkaConsumerGroupOps<'a> {
    pub fn new(
        admin: &'a rdkafka::admin::AdminClient<rdkafka::client::DefaultClientContext>,
        consumer: &'a Arc<BaseConsumer>,
        config: &'a ClusterConfig,
    ) -> Self {
        Self { admin, consumer, config }
    }

    /// Create a temporary consumer configured with the target group.id
    /// This allows fetching committed offsets for any consumer group
    fn create_group_consumer(&self, group_id: &str) -> std::result::Result<BaseConsumer, KafkaClientError> {
        let mut consumer_config = self.config.to_rdkafka_config();
        consumer_config.set("group.id", group_id);
        consumer_config.set("enable.auto.commit", "false");
        consumer_config
            .create()
            .map_err(|e| KafkaClientError::Connection(format!("Failed to create group consumer: {}", e)))
    }

    pub async fn list_groups(&self) -> Result<Vec<String>> {
        let groups = self
            .admin
            .inner()
            .fetch_group_list(None, std::time::Duration::from_secs(5))
            .map_err(|e| KafkaClientError::Connection(format!("Failed to fetch groups: {}", e)))?;

        Ok(groups
            .groups()
            .iter()
            .map(|g| g.name().to_string())
            .collect())
    }

    pub async fn get_group_info(&self, group_id: &str) -> Result<ConsumerGroupInfo> {
        let groups = self
            .admin
            .inner()
            .fetch_group_list(Some(group_id), std::time::Duration::from_secs(5))
            .map_err(|e| KafkaClientError::Connection(format!("Failed to fetch group info: {}", e)))?;

        let group = groups
            .groups()
            .first()
            .ok_or_else(|| KafkaClientError::ConsumerGroupNotFound(group_id.to_string()))?;

        // Parse member assignments to extract partition assignments
        let mut member_partition_map: std::collections::HashMap<String, Vec<(String, i32)>> =
            std::collections::HashMap::new();

        let members: Vec<ConsumerGroupMember> = group
            .members()
            .iter()
            .map(|m| {
                let member_id = m.id().to_string();
                let mut assignment_partitions: Vec<i32> = vec![];
                let mut assigned_topic_partitions: Vec<(String, i32)> = vec![];

                // Parse assignment data if present
                if let Some(assignment_data) = m.assignment() {
                    if !assignment_data.is_empty() {
                        assigned_topic_partitions =
                            self.parse_assignment_data(assignment_data);
                        assignment_partitions = assigned_topic_partitions
                            .iter()
                            .map(|(_, p)| *p)
                            .collect();
                    }
                }

                member_partition_map.insert(member_id.clone(), assigned_topic_partitions);

                ConsumerGroupMember {
                    member_id,
                    client_id: m.client_id().to_string(),
                    client_host: m.client_host().to_string(),
                    assignment: assignment_partitions,
                }
            })
            .collect();

        // Get all topics from member assignments
        let mut subscribed_topics: std::collections::HashSet<String> = std::collections::HashSet::new();
        for tps in member_partition_map.values() {
            for (topic, _) in tps {
                subscribed_topics.insert(topic.clone());
            }
        }

        let mut partitions: Vec<GroupPartitionInfo> = Vec::new();

        // Create a consumer with the target group.id to fetch committed offsets
        let group_consumer = self.create_group_consumer(group_id);

        // For each subscribed topic, build partition info
        for topic_name in &subscribed_topics {
            let topic_metadata = self
                .admin
                .inner()
                .fetch_metadata(Some(topic_name), std::time::Duration::from_secs(5))
                .map_err(|e| KafkaClientError::Connection(format!("Failed to fetch metadata: {}", e)))?;

            if let Some(topic_meta) = topic_metadata.topics().first() {
                    // Build TopicPartitionList for committed offset query
                    use rdkafka::TopicPartitionList;
                    let mut tpl = TopicPartitionList::new();
                    for partition in topic_meta.partitions() {
                        tpl.add_partition(topic_name, partition.id());
                    }

                    // Fetch committed offsets using the group-specific consumer
                    let committed_result = if let Ok(ref gc) = group_consumer {
                        gc.committed_offsets(tpl.clone(), std::time::Duration::from_secs(5))
                            .ok()
                    } else {
                        None
                    };

                    for partition in topic_meta.partitions() {
                        let partition_id = partition.id();

                        // Get committed offset from the result
                        let current_offset = match &committed_result {
                            Some(offsets) => {
                                match offsets.find_partition(topic_name, partition_id) {
                                    Some(p) => p.offset().to_raw().unwrap_or(-1),
                                    None => -1,
                                }
                            }
                            None => -1,
                        };

                        // Get end offset (high watermark)
                        let end_offset = self
                            .consumer
                            .fetch_watermarks(topic_name, partition_id, std::time::Duration::from_secs(5))
                            .map(|(_, high)| high)
                            .unwrap_or(-1);

                        let lag = if current_offset >= 0 && end_offset >= 0 {
                            end_offset - current_offset
                        } else {
                            -1
                        };

                        // Find member assigned to this partition
                        let member_id = member_partition_map.iter().find_map(|(mid, tps)| {
                            if tps.iter().any(|(t, p)| t == topic_name && *p == partition_id) {
                                Some(mid.clone())
                            } else {
                                None
                            }
                        });

                        partitions.push(GroupPartitionInfo {
                            topic: topic_name.clone(),
                            partition: partition_id,
                            current_offset,
                            end_offset,
                            lag,
                            member_id,
                        });
                    }
                }
        }

        Ok(ConsumerGroupInfo {
            group_id: group.name().to_string(),
            state: format!("{:?}", group.state()),
            members,
            partitions,
        })
    }

    /// Parse assignment data from member metadata
    /// The Kafka consumer protocol assignment format:
    /// - Version (int16, big-endian)
    /// - Topics array length (int32, big-endian)
    /// - For each topic:
    ///   - Topic name (string: int16 length + bytes)
    ///   - Partitions array (int32 length + int32 values)
    /// - User data (bytes, nullable)
    fn parse_assignment_data(&self, data: &[u8]) -> Vec<(String, i32)> {
        let mut result = Vec::new();
        let mut offset = 0;

        // Need at least 2 bytes for version
        if data.len() < 2 {
            return result;
        }

        // Read version (int16)
        let _version = i16::from_be_bytes([data[0], data[1]]);
        offset += 2;

        // Read topics array length
        if offset + 4 > data.len() {
            return result;
        }
        let topics_count = i32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Guard against negative counts (malformed data)
        let topics_count = topics_count.max(0) as usize;

        for _ in 0..topics_count {
            // Read topic name (string: int16 length + bytes)
            if offset + 2 > data.len() {
                break;
            }
            let topic_len = i16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            if offset + topic_len > data.len() {
                break;
            }
            let topic_name = String::from_utf8_lossy(&data[offset..offset + topic_len]).to_string();
            offset += topic_len;

            // Read partitions array length
            if offset + 4 > data.len() {
                break;
            }
            let partitions_count = i32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            // Guard against negative counts (malformed data)
            let partitions_count = partitions_count.max(0) as usize;

            for _ in 0..partitions_count {
                if offset + 4 > data.len() {
                    break;
                }
                let partition = i32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                offset += 4;
                result.push((topic_name.clone(), partition));
            }
        }

        result
    }

    pub async fn get_group_lag(&self, group_id: &str) -> Result<Vec<LagInfo>> {
        // Fetch group metadata to get subscribed topics
        let groups = self
            .admin
            .inner()
            .fetch_group_list(Some(group_id), std::time::Duration::from_secs(5))
            .map_err(|e| KafkaClientError::Connection(format!("Failed to fetch group: {}", e)))?;

        let group = groups
            .groups()
            .first()
            .ok_or_else(|| KafkaClientError::ConsumerGroupNotFound(group_id.to_string()))?;

        // Collect topics from member assignments
        let mut topics: std::collections::HashSet<String> = std::collections::HashSet::new();
        for member in group.members() {
            if let Some(assignment) = member.assignment() {
                if !assignment.is_empty() {
                    let parsed = self.parse_assignment_data(assignment);
                    for (topic_name, _) in parsed {
                        topics.insert(topic_name);
                    }
                }
            }
        }

        let mut lag_info = Vec::new();

        // Create a consumer with the target group.id to fetch committed offsets
        let group_consumer = self.create_group_consumer(group_id);

        // For each topic, get partition info with end offsets
        for topic_name in topics {
            let topic_metadata = self
                .admin
                .inner()
                .fetch_metadata(Some(&topic_name), std::time::Duration::from_secs(5))
                .map_err(|e| KafkaClientError::Connection(format!("Failed to fetch metadata: {}", e)))?;

            if let Some(topic_meta) = topic_metadata.topics().first() {
                // Build TopicPartitionList for committed offset query
                use rdkafka::TopicPartitionList;
                let mut tpl = TopicPartitionList::new();
                for partition in topic_meta.partitions() {
                    tpl.add_partition(&topic_name, partition.id());
                }

                // Fetch committed offsets using the group-specific consumer
                let committed_result = if let Ok(ref gc) = group_consumer {
                    gc.committed_offsets(tpl, std::time::Duration::from_secs(5))
                        .ok()
                } else {
                    None
                };

                for partition in topic_meta.partitions() {
                    let partition_id = partition.id();

                    // Get committed offset from the result
                    let current_offset = match &committed_result {
                        Some(offsets) => {
                            match offsets.find_partition(&topic_name, partition_id) {
                                Some(p) => p.offset().to_raw().unwrap_or(-1),
                                None => -1,
                            }
                        }
                        None => -1,
                    };

                    // Get end offset (high watermark)
                    let end_offset = self
                        .consumer
                        .fetch_watermarks(&topic_name, partition_id, std::time::Duration::from_secs(5))
                        .map(|(_, high)| high)
                        .unwrap_or(-1);

                    let lag = if current_offset >= 0 && end_offset >= 0 {
                        end_offset - current_offset
                    } else {
                        -1
                    };

                    lag_info.push(LagInfo {
                        topic: topic_name.clone(),
                        partition: partition_id,
                        current_offset,
                        end_offset,
                        lag,
                    });
                }
            }
        }

        Ok(lag_info)
    }

    /// Reset consumer group offsets for a topic
    pub async fn reset_offsets(
        &self,
        group_id: &str,
        topic: &str,
        partitions: Vec<i32>,
        reset_type: crate::models::ResetOffsetType,
    ) -> Result<()> {
        use crate::models::ResetOffsetType;
        use rdkafka::consumer::CommitMode;
        use rdkafka::TopicPartitionList;

        // Determine partitions to reset
        let partitions_to_reset = if partitions.is_empty() {
            // Get all partitions for the topic
            let metadata = self
                .admin
                .inner()
                .fetch_metadata(Some(topic), std::time::Duration::from_secs(5))
                .map_err(|e| KafkaClientError::Connection(format!("Failed to fetch metadata: {}", e)))?;

            let topic_meta = metadata
                .topics()
                .first()
                .ok_or_else(|| KafkaClientError::TopicNotFound(topic.to_string()))?;

            topic_meta.partitions().iter().map(|p| p.id()).collect()
        } else {
            partitions
        };

        // Create a consumer with the target group.id
        let group_consumer = self.create_group_consumer(group_id)?;

        // Build TopicPartitionList with offsets
        let mut tpl = TopicPartitionList::new();
        for &partition in &partitions_to_reset {
            let offset = match &reset_type {
                ResetOffsetType::Earliest => rdkafka::Offset::Beginning,
                ResetOffsetType::Latest => rdkafka::Offset::End,
                ResetOffsetType::Timestamp(_) => {
                    // Timestamp-based reset requires converting timestamp to offset via offsets_for_times()
                    // This is not yet implemented. Return an error for now.
                    return Err(KafkaClientError::Connection(
                        "Timestamp-based offset reset is not yet implemented. Use 'earliest', 'latest', or 'specific' instead.".to_string()
                    ));
                }
                ResetOffsetType::Specific(offset) => rdkafka::Offset::Offset(*offset),
            };
            tpl.add_partition_offset(topic, partition, offset)
                .map_err(|e| KafkaClientError::Connection(format!("Failed to add partition offset: {}", e)))?;
        }

        // Commit the offsets to reset them
        group_consumer
            .commit(&tpl, CommitMode::Sync)
            .map_err(|e| KafkaClientError::Connection(format!("Failed to commit offsets: {}", e)))?;

        Ok(())
    }
}