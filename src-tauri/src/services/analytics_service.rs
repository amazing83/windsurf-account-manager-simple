use crate::utils::{AppError, AppResult};
use std::sync::Arc;

const WINDSURF_BASE_URL: &str = "https://web-backend.windsurf.com";

pub struct AnalyticsService {
    client: Arc<reqwest::Client>,
}

impl AnalyticsService {
    pub fn new() -> Self {
        // 使用全局共享的 HTTP 客户端，避免每次请求都创建新实例
        Self {
            client: super::get_http_client(),
        }
    }

    /// 构建 GetAnalytics 请求体
    ///
    /// 请求格式:
    /// - field 2 (repeated QueryRequest): 查询请求数组
    /// - field 3 (start_timestamp): 开始时间
    /// - field 4 (end_timestamp): 结束时间
    /// - field 5 (api_key): API密钥
    ///
    /// QueryRequest 字段编号:
    /// - 1: completion_stats
    /// - 2: completions_by_day
    /// - 3: completions_by_language
    /// - 10: chats_by_day
    /// - 12: chats_by_model
    /// - 15: percent_code_written
    /// - 17: chat_stats
    /// - 23: cascade_lines
    /// - 24: cascade_tool_usage
    /// - 25: cascade_runs
    /// - 29: daily_active_user_counts
    fn build_get_analytics_body(&self, api_key: &str, start_timestamp: i64, end_timestamp: i64, is_team: bool) -> Vec<u8> {
        let mut body = Vec::new();

        // ===== Cascade 相关查询（已验证可用）=====
        
        // Field 2: QueryRequest - cascade_lines (field 23 in QueryRequest)
        // Tag: (23 << 3) | 2 = 0xBA 0x01, empty message = 0x00
        body.extend_from_slice(&[0x12, 0x03, 0xBA, 0x01, 0x00]);

        // Field 2: QueryRequest - cascade_tool_usage (field 24 in QueryRequest)
        // Tag: (24 << 3) | 2 = 0xC2 0x01
        body.extend_from_slice(&[0x12, 0x03, 0xC2, 0x01, 0x00]);

        // Field 2: QueryRequest - cascade_runs (field 25 in QueryRequest)
        // Tag: (25 << 3) | 2 = 0xCA 0x01
        body.extend_from_slice(&[0x12, 0x03, 0xCA, 0x01, 0x00]);

        // ===== 新增查询（已验证可用）=====
        // completion_stats (field 1) - 团队✓ 普通✓
        body.extend_from_slice(&[0x12, 0x02, 0x0A, 0x00]);
        // completions_by_language (field 3) - 团队✓ 普通✓
        body.extend_from_slice(&[0x12, 0x02, 0x1A, 0x00]);
        // chats_by_model (field 12) - 团队✓ 普通✓
        body.extend_from_slice(&[0x12, 0x02, 0x62, 0x00]);
        
        // ===== 带 time_zone 参数的查询 =====
        // completions_by_day (field 2) - 需要 time_zone 参数
        // QueryRequest tag: 0x12, inner field 2 tag: 0x12
        // QueryRequestCompletionsByDay { time_zone: "Asia/Shanghai" }
        let tz = b"Asia/Shanghai";
        let tz_msg_len = 2 + tz.len(); // 0x0A + len + tz_bytes
        let query_msg_len = 2 + tz_msg_len; // 0x12 + len + inner_msg
        body.push(0x12); // QueryRequest tag
        body.push(query_msg_len as u8);
        body.push(0x12); // completions_by_day field tag (2 << 3 | 2)
        body.push(tz_msg_len as u8);
        body.push(0x0A); // time_zone field tag (1 << 3 | 2)
        body.push(tz.len() as u8);
        body.extend_from_slice(tz);

        // chats_by_day (field 10) - 需要 time_zone 参数
        // QueryRequest tag: 0x12, inner field 10 tag: 0x52
        let query_msg_len = 2 + tz_msg_len;
        body.push(0x12); // QueryRequest tag
        body.push(query_msg_len as u8);
        body.push(0x52); // chats_by_day field tag (10 << 3 | 2)
        body.push(tz_msg_len as u8);
        body.push(0x0A); // time_zone field tag
        body.push(tz.len() as u8);
        body.extend_from_slice(tz);

        // ===== 仅团队账户可用的查询 =====
        if is_team {
            // percent_code_written (field 15) - 仅团队账户可用
            body.extend_from_slice(&[0x12, 0x02, 0x7A, 0x00]);
        }

        // Field 3: start_timestamp (google.protobuf.Timestamp)
        // Tag: (3 << 3) | 2 = 0x1A
        body.push(0x1A);
        let start_ts_bytes = Self::encode_timestamp(start_timestamp);
        body.push(start_ts_bytes.len() as u8);
        body.extend_from_slice(&start_ts_bytes);

        // Field 4: end_timestamp (google.protobuf.Timestamp)
        // Tag: (4 << 3) | 2 = 0x22
        body.push(0x22);
        let end_ts_bytes = Self::encode_timestamp(end_timestamp);
        body.push(end_ts_bytes.len() as u8);
        body.extend_from_slice(&end_ts_bytes);

        // Field 5: api_key (string)
        // Tag: (5 << 3) | 2 = 0x2A
        body.push(0x2A);
        let api_key_bytes = api_key.as_bytes();
        body.push(api_key_bytes.len() as u8);
        body.extend_from_slice(api_key_bytes);

        body
    }

    /// 编码 Timestamp 为 Protobuf 格式
    /// Timestamp 包含两个字段:
    /// - field 1: seconds (int64)
    /// - field 2: nanos (int32) - 通常为0
    fn encode_timestamp(timestamp: i64) -> Vec<u8> {
        let mut ts_body = Vec::new();
        
        // Field 1: seconds
        // Tag: (1 << 3) | 0 = 0x08
        ts_body.push(0x08);
        ts_body.extend_from_slice(&Self::encode_varint(timestamp as u64));
        
        ts_body
    }

    /// 编码 varint
    fn encode_varint(mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            result.push(byte);
            if value == 0 {
                break;
            }
        }
        result
    }

    /// 调用 GetAnalytics API
    /// is_team: 是否是团队账户，团队账户会额外请求 percent_code_written
    pub async fn get_analytics(&self, api_key: &str, start_timestamp: i64, end_timestamp: i64, is_team: bool) -> AppResult<Vec<u8>> {
        let url = format!("{}/exa.user_analytics_pb.UserAnalyticsService/GetAnalytics", WINDSURF_BASE_URL);
        
        let body = self.build_get_analytics_body(api_key, start_timestamp, end_timestamp, is_team);

        println!("[GetAnalytics] Calling API with time range: {} - {}", start_timestamp, end_timestamp);
        println!("[GetAnalytics] Request body length: {} bytes", body.len());
        println!("[GetAnalytics] Request body hex: {}", body.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        
        let response = self.client
            .post(&url)
            .header("accept", "*/*")
            .header("accept-language", "zh-CN,zh;q=0.9")
            .header("cache-control", "no-cache")
            .header("connect-protocol-version", "1")
            .header("content-type", "application/proto")
            .header("pragma", "no-cache")
            .header("sec-ch-ua", "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .header("x-auth-token", api_key)
            .header("x-api-key", api_key)
            .header("x-debug-email", "")
            .header("x-debug-team-name", "")
            .header("Referer", "https://windsurf.com/")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::Api(format!("Request failed: {}", e)))?;
        
        let status_code = response.status().as_u16();
        let response_body = response.bytes().await
            .map_err(|e| AppError::Api(format!("Failed to read response: {}", e)))?
            .to_vec();
        
        println!("[GetAnalytics] Response status: {}", status_code);
        println!("[GetAnalytics] Response body length: {} bytes", response_body.len());

        if status_code != 200 {
            // 尝试解析错误响应
            if let Ok(error_text) = String::from_utf8(response_body.clone()) {
                println!("[GetAnalytics] Error response: {}", error_text);
                return Err(AppError::Api(format!("API returned status code {}: {}", status_code, error_text)));
            }
            return Err(AppError::Api(format!("API returned status code: {}", status_code)));
        }
        
        Ok(response_body)
    }
}

