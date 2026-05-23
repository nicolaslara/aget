use crate::browser_cdp::client::cdp_websocket_config;

#[test]
fn browser_cdp_websocket_config_allows_large_frames() {
    let config = cdp_websocket_config();

    assert_eq!(config.max_message_size, None);
    assert_eq!(config.max_frame_size, None);
}
