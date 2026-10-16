#![allow(clippy::module_inception)]

pub mod event {
    include!("kytrix.event.rs");
}

pub mod client {
    include!("kytrix.client.rs");
}

pub mod server {
    include!("kytrix.server.rs");
}

pub mod topic {
    include!("kytrix.topic.rs");
}

pub mod types {
    include!("kytrix.types.rs");
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use prost::Message as _;

    use crate::{client, server, topic, types};

    #[test]
    fn rate_limit_error_response_should_round_trip() {
        let response = server::Response {
            id: 42,
            response: Some(server::response::Response::Error(server::ErrorResponse {
                code: server::ErrorCode::RateLimit as i32,
            })),
        };

        assert_eq!(
            server::Response::decode(response.encode_to_vec().as_slice()).unwrap(),
            response
        );
    }

    #[test]
    fn tweet_wire_contract_should_use_contiguous_field_numbers() {
        let tweet = types::Tweet {
            tweet_id: "1".to_owned(),
            timestamp: 1,
            text_segments: vec![types::TweetTextSegment {
                text: "x".to_owned(),
                url: Some("u".to_owned()),
            }],
            reference: Some(types::tweet::Reference::Quote(Box::default())),
            x_user_id: "2".to_owned(),
            media: vec![types::TweetMedia { content: None }],
        };

        assert_eq!(
            tweet.encode_to_vec(),
            [
                0x0a, 0x01, b'1', 0x11, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a, 0x06,
                0x0a, 0x01, b'x', 0x12, 0x01, b'u', 0x2a, 0x00, 0x32, 0x01, b'2', 0x3a, 0x00,
            ]
        );
        assert_eq!(
            types::Tweet {
                reference: Some(types::tweet::Reference::Repost(Box::default())),
                ..types::Tweet::default()
            }
            .encode_to_vec(),
            [0x22, 0x00]
        );
    }

    #[test]
    fn tweet_text_segment_wire_contract_should_preserve_optional_url_presence() {
        let absent = types::TweetTextSegment {
            text: "x".to_owned(),
            url: None,
        };
        let present_empty = types::TweetTextSegment {
            text: "x".to_owned(),
            url: Some(String::new()),
        };

        assert_eq!(absent.encode_to_vec(), [0x0a, 0x01, b'x']);
        assert_eq!(
            present_empty.encode_to_vec(),
            [0x0a, 0x01, b'x', 0x12, 0x00]
        );
    }

    #[test]
    fn websocket_reauthentication_messages_should_round_trip() {
        let request = client::Request {
            id: 17,
            request: Some(client::request::Request::Reauthenticate(
                client::Reauthenticate {
                    access_token: "access-token".to_owned(),
                    dpop_proof: "proof".to_owned(),
                },
            )),
        };
        let challenge = server::ServerMessage {
            message: Some(server::server_message::Message::AuthChallenge(
                server::AuthChallenge {
                    nonce: "nonce".to_owned(),
                    expires_at: 42,
                },
            )),
        };
        let response = server::Response {
            id: request.id,
            response: Some(server::response::Response::Reauthenticate(
                server::ReauthenticateResponse {},
            )),
        };

        assert_eq!(
            client::Request::decode(request.encode_to_vec().as_slice()).unwrap(),
            request
        );
        assert_eq!(
            server::ServerMessage::decode(challenge.encode_to_vec().as_slice()).unwrap(),
            challenge
        );
        assert_eq!(
            server::Response::decode(response.encode_to_vec().as_slice()).unwrap(),
            response
        );
        assert_eq!(
            server::ErrorCode::try_from(server::ErrorCode::AuthenticationUnavailable as i32),
            Ok(server::ErrorCode::AuthenticationUnavailable)
        );
    }

    #[test]
    fn business_rpc_requests_and_responses_should_round_trip() {
        let requests = vec![
            client::request::Request::GetEtf(client::GetEtf {}),
            client::request::Request::GetIndexManifest(client::GetIndexManifest {}),
            client::request::Request::XSearch(client::XSearch {
                query: "kytrix".to_owned(),
            }),
            client::request::Request::XSelections(client::XSelections {}),
            client::request::Request::XAddSelection(client::XAddSelection {
                username: "kytrix".to_owned(),
            }),
            client::request::Request::XDeleteSelection(client::XDeleteSelection {
                x_user_id: "42".to_owned(),
            }),
            client::request::Request::XTweets(client::XTweets {}),
            client::request::Request::WalletSelections(client::WalletSelections {}),
            client::request::Request::WalletAddSelection(client::WalletAddSelection {
                address: "0x0000000000000000000000000000000000000001".to_owned(),
            }),
            client::request::Request::WalletDeleteSelection(client::WalletDeleteSelection {
                address: "11111111111111111111111111111111".to_owned(),
            }),
        ];
        for (offset, request) in requests.into_iter().enumerate() {
            let request = client::Request {
                id: u32::try_from(offset + 1).unwrap(),
                request: Some(request),
            };
            assert_eq!(
                client::Request::decode(request.encode_to_vec().as_slice()).unwrap(),
                request
            );
        }

        let selection = types::XSelection {
            x_user_id: "42".to_owned(),
            status: types::XSelectionStatus::XActive as i32,
            created_at: 1,
            updated_at: 2,
            profile: Some(types::XProfile {
                x_user_id: "42".to_owned(),
                username: "kytrix".to_owned(),
                display_name: "Kytrix".to_owned(),
                avatar_url: "https://example.test/avatar.jpg".to_owned(),
                verified: true,
                revision: 7,
                description: "profile".to_owned(),
                followers_count: 10,
                following_count: 2,
                protected: false,
            }),
        };
        let response = server::Response {
            id: 17,
            response: Some(server::response::Response::XSelections(
                server::XSelectionsResponse {
                    selections: vec![selection],
                    x_account_limit: 3,
                },
            )),
        };
        assert_eq!(
            server::Response::decode(response.encode_to_vec().as_slice()).unwrap(),
            response
        );
    }

    #[test]
    fn illegal_websocket_messages_and_enum_values_should_be_rejected() {
        assert!(client::Request::decode([0xff].as_slice()).is_err());
        assert!(server::ServerMessage::decode([0x0a, 0x02, 0x08].as_slice()).is_err());

        let empty_request = client::Request::decode([].as_slice()).unwrap();
        assert_eq!((empty_request.id, empty_request.request), (0, None));
        assert!(server::ErrorCode::try_from(i32::MAX).is_err());
        assert!(types::XSelectionStatus::try_from(i32::MAX).is_err());
    }

    #[test]
    fn every_typed_rpc_error_code_should_round_trip() {
        for code in [
            server::ErrorCode::InvalidRequest,
            server::ErrorCode::NotReady,
            server::ErrorCode::InvalidArgument,
            server::ErrorCode::NotFound,
            server::ErrorCode::QuotaExceeded,
            server::ErrorCode::ProtectedAccount,
            server::ErrorCode::UpstreamUnavailable,
            server::ErrorCode::Timeout,
            server::ErrorCode::MutationUnknown,
            server::ErrorCode::Protocol,
        ] {
            let response = server::Response {
                id: 1,
                response: Some(server::response::Response::Error(server::ErrorResponse {
                    code: code as i32,
                })),
            };
            let decoded = server::Response::decode(response.encode_to_vec().as_slice()).unwrap();
            let Some(server::response::Response::Error(error)) = decoded.response else {
                panic!("typed error response should retain its oneof")
            };
            assert_eq!(server::ErrorCode::try_from(error.code), Ok(code));
        }
    }

    #[test]
    fn final_topic_set_and_session_state_should_round_trip() {
        let topics = vec![
            topic::Topic {
                topic: Some(topic::topic::Topic::News(topic::NewsTopic {})),
            },
            topic::Topic {
                topic: Some(topic::topic::Topic::Index(topic::IndexTopic {
                    code: "SPX".to_owned(),
                })),
            },
        ];
        let request = client::SetSubscriptions {
            topics: topics.clone(),
        };
        let response = server::SetSubscriptionsResponse {
            topics: topics.clone(),
        };
        assert_eq!(
            client::SetSubscriptions::decode(request.encode_to_vec().as_slice()).unwrap(),
            request
        );
        assert_eq!(
            server::SetSubscriptionsResponse::decode(response.encode_to_vec().as_slice()).unwrap(),
            response
        );
        let state = server::SessionState {
            entitlement_revision: 7,
            news_read: true,
            index_topic_limit: 0,
            next_change_at: Some(42),
            wallet_address_limit: 3,
        };
        assert_eq!(
            server::SessionState::decode(state.encode_to_vec().as_slice()).unwrap(),
            state
        );
    }

    #[test]
    fn complete_index_manifest_should_round_trip() {
        let manifest = types::IndexManifest {
            categories: vec![types::IndexCategory {
                code: "asia".to_owned(),
                name: Some(types::LocalizedName {
                    en: "Asian Stock Markets".to_owned(),
                    zh: "亚洲股市".to_owned(),
                }),
            }],
            indices: vec![types::IndexDefinition {
                request_code: ".HSI".to_owned(),
                code: "HSI".to_owned(),
                category: "asia".to_owned(),
                name: Some(types::LocalizedName {
                    en: "Hang Seng Index".to_owned(),
                    zh: "恒生指数".to_owned(),
                }),
            }],
        };

        assert_eq!(
            types::IndexManifest::decode(manifest.encode_to_vec().as_slice()).unwrap(),
            manifest
        );
    }

    #[test]
    fn etf_and_index_manifest_json_should_use_rust_field_names_and_round_trip() {
        let etf = types::Etf {
            btc: HashMap::from([(
                "2026-08-08".to_owned(),
                types::etf::Btc {
                    ibit: "1".to_owned(),
                    fbtc: "2".to_owned(),
                    ..Default::default()
                },
            )]),
            eth: HashMap::from([(
                "2026-08-08".to_owned(),
                types::etf::Eth {
                    etha: "3".to_owned(),
                    ..Default::default()
                },
            )]),
            sol: HashMap::from([(
                "2026-08-08".to_owned(),
                types::etf::Sol {
                    bsol: "4".to_owned(),
                    ..Default::default()
                },
            )]),
            hype: HashMap::from([(
                "2026-08-08".to_owned(),
                types::etf::Hype {
                    bhyp: "5".to_owned(),
                    ..Default::default()
                },
            )]),
            timestamp: 42,
        };
        let manifest = types::IndexManifest {
            categories: vec![types::IndexCategory {
                code: "asia".to_owned(),
                name: Some(types::LocalizedName {
                    en: "Asia".to_owned(),
                    zh: "亚洲".to_owned(),
                }),
            }],
            indices: vec![types::IndexDefinition {
                request_code: ".HSI".to_owned(),
                code: "HSI".to_owned(),
                category: "asia".to_owned(),
                name: Some(types::LocalizedName {
                    en: "Hang Seng Index".to_owned(),
                    zh: "恒生指数".to_owned(),
                }),
            }],
        };

        let etf_json = serde_json::to_string(&etf).unwrap();
        let manifest_json = serde_json::to_string(&manifest).unwrap();
        assert!(etf_json.contains(r#""ibit":"1""#));
        assert!(!etf_json.contains("IBIT"));
        assert!(manifest_json.contains(r#""request_code":".HSI""#));
        assert_eq!(serde_json::from_str::<types::Etf>(&etf_json).unwrap(), etf);
        assert_eq!(
            serde_json::from_str::<types::IndexManifest>(&manifest_json).unwrap(),
            manifest
        );
    }
}
