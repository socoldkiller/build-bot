use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NapCatResponse {
    #[serde(rename = "self_id", default)]
    pub self_id: i64,
    #[serde(rename = "user_id", default)]
    pub user_id: i64,
    #[serde(rename = "group_id", default)]
    pub group_id: i64,
    #[serde(rename = "time", default)]
    pub time: i64,
    #[serde(rename = "message_id", default)]
    pub message_id: i64,
    #[serde(rename = "message_seq", default)]
    pub message_seq: i64,
    #[serde(rename = "real_id", default)]
    pub real_id: i64,
    #[serde(rename = "real_seq", default)]
    pub real_seq: String,
    #[serde(rename = "message_type", default)]
    pub message_type: String,
    #[serde(default)]
    pub sender: Sender,
    #[serde(rename = "raw_message", default)]
    pub raw_message: String,
    #[serde(default)]
    pub font: i64,
    #[serde(rename = "sub_type", default)]
    pub sub_type: String,
    #[serde(default)]
    pub message: Vec<Message>,
    #[serde(rename = "message_format", default)]
    pub message_format: String,
    #[serde(rename = "post_type", default)]
    pub post_type: String,
    #[serde(rename = "target_id", default)]
    pub target_id: i64,
    #[serde(default)]
    pub raw: Raw,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Sender {
    #[serde(rename = "user_id", default)]
    pub user_id: i64,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub card: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Data {
    #[serde(default)]
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Message {
    #[serde(rename = "type", default)]
    pub msg_type: String,
    #[serde(default)]
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MsgMeta {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExtBufForUI {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TextElement {
    #[serde(default)]
    pub content: String,
    #[serde(rename = "atType", default)]
    pub at_type: i64,
    #[serde(rename = "atUid", default)]
    pub at_uid: String,
    #[serde(rename = "atTinyId", default)]
    pub at_tiny_id: String,
    #[serde(rename = "atNtUid", default)]
    pub at_nt_uid: String,
    #[serde(rename = "subElementType", default)]
    pub sub_element_type: i64,
    #[serde(rename = "atChannelId", default)]
    pub at_channel_id: String,
    #[serde(rename = "linkInfo", default)]
    pub link_info: Value,
    #[serde(rename = "atRoleId", default)]
    pub at_role_id: String,
    #[serde(rename = "atRoleColor", default)]
    pub at_role_color: i64,
    #[serde(rename = "atRoleName", default)]
    pub at_role_name: String,
    #[serde(rename = "needNotify", default)]
    pub need_notify: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Elements {
    #[serde(rename = "elementType", default)]
    pub element_type: i64,
    #[serde(rename = "elementId", default)]
    pub element_id: String,
    #[serde(rename = "elementGroupId", default)]
    pub element_group_id: i64,
    #[serde(rename = "extBufForUI", default)]
    pub ext_buf_for_ui: ExtBufForUI,
    #[serde(rename = "textElement", default)]
    pub text_element: TextElement,

    #[serde(rename = "faceElement", default)]
    pub face_element: Value,
    #[serde(rename = "marketFaceElement", default)]
    pub market_face_element: Value,
    #[serde(rename = "replyElement", default)]
    pub reply_element: Value,
    #[serde(rename = "picElement", default)]
    pub pic_element: Value,
    #[serde(rename = "pttElement", default)]
    pub ptt_element: Value,
    #[serde(rename = "videoElement", default)]
    pub video_element: Value,
    #[serde(rename = "grayTipElement", default)]
    pub gray_tip_element: Value,
    #[serde(rename = "arkElement", default)]
    pub ark_element: Value,
    #[serde(rename = "fileElement", default)]
    pub file_element: Value,
    #[serde(rename = "liveGiftElement", default)]
    pub live_gift_element: Value,
    #[serde(rename = "markdownElement", default)]
    pub markdown_element: Value,
    #[serde(rename = "structLongMsgElement", default)]
    pub struct_long_msg_element: Value,
    #[serde(rename = "multiForwardMsgElement", default)]
    pub multi_forward_msg_element: Value,
    #[serde(rename = "giphyElement", default)]
    pub giphy_element: Value,
    #[serde(rename = "walletElement", default)]
    pub wallet_element: Value,
    #[serde(rename = "inlineKeyboardElement", default)]
    pub inline_keyboard_element: Value,
    #[serde(rename = "textGiftElement", default)]
    pub text_gift_element: Value,
    #[serde(rename = "calendarElement", default)]
    pub calendar_element: Value,
    #[serde(rename = "yoloGameResultElement", default)]
    pub yolo_game_result_element: Value,
    #[serde(rename = "avRecordElement", default)]
    pub av_record_element: Value,
    #[serde(rename = "structMsgElement", default)]
    pub struct_msg_element: Value,
    #[serde(rename = "faceBubbleElement", default)]
    pub face_bubble_element: Value,
    #[serde(rename = "shareLocationElement", default)]
    pub share_location_element: Value,
    #[serde(rename = "tofuRecordElement", default)]
    pub tofu_record_element: Value,
    #[serde(rename = "taskTopMsgElement", default)]
    pub task_top_msg_element: Value,
    #[serde(rename = "recommendedMsgElement", default)]
    pub recommended_msg_element: Value,
    #[serde(rename = "actionBarElement", default)]
    pub action_bar_element: Value,
    #[serde(rename = "prologueMsgElement", default)]
    pub prologue_msg_element: Value,
    #[serde(rename = "forwardMsgElement", default)]
    pub forward_msg_element: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FromChannelRoleInfo {
    #[serde(rename = "roleId", default)]
    pub role_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FromGuildRoleInfo {
    #[serde(rename = "roleId", default)]
    pub role_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LevelRoleInfo {
    #[serde(rename = "roleId", default)]
    pub role_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub color: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GeneralFlags {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MsgAttrs {}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Raw {
    #[serde(rename = "msgId", default)]
    pub msg_id: String,
    #[serde(rename = "msgRandom", default)]
    pub msg_random: String,
    #[serde(rename = "msgSeq", default)]
    pub msg_seq: String,
    #[serde(rename = "cntSeq", default)]
    pub cnt_seq: String,
    #[serde(rename = "chatType", default)]
    pub chat_type: i64,
    #[serde(rename = "msgType", default)]
    pub msg_type: i64,
    #[serde(rename = "subMsgType", default)]
    pub sub_msg_type: i64,
    #[serde(rename = "sendType", default)]
    pub send_type: i64,
    #[serde(rename = "senderUid", default)]
    pub sender_uid: String,
    #[serde(rename = "peerUid", default)]
    pub peer_uid: String,
    #[serde(rename = "channelId", default)]
    pub channel_id: String,
    #[serde(rename = "guildId", default)]
    pub guild_id: String,
    #[serde(rename = "guildCode", default)]
    pub guild_code: String,
    #[serde(rename = "fromUid", default)]
    pub from_uid: String,
    #[serde(rename = "fromAppid", default)]
    pub from_appid: String,
    #[serde(rename = "msgTime", default)]
    pub msg_time: String,
    #[serde(rename = "msgMeta", default)]
    pub msg_meta: MsgMeta,
    #[serde(rename = "sendStatus", default)]
    pub send_status: i64,
    #[serde(rename = "sendRemarkName", default)]
    pub send_remark_name: String,
    #[serde(rename = "sendMemberName", default)]
    pub send_member_name: String,
    #[serde(rename = "sendNickName", default)]
    pub send_nick_name: String,
    #[serde(rename = "guildName", default)]
    pub guild_name: String,
    #[serde(rename = "channelName", default)]
    pub channel_name: String,
    #[serde(default)]
    pub elements: Vec<Elements>,
    #[serde(default)]
    pub records: Vec<Value>,
    #[serde(rename = "emojiLikesList", default)]
    pub emoji_likes_list: Vec<Value>,
    #[serde(rename = "commentCnt", default)]
    pub comment_cnt: String,
    #[serde(rename = "directMsgFlag", default)]
    pub direct_msg_flag: i64,
    #[serde(rename = "directMsgMembers", default)]
    pub direct_msg_members: Vec<Value>,
    #[serde(rename = "peerName", default)]
    pub peer_name: String,
    #[serde(rename = "freqLimitInfo", default)]
    pub freq_limit_info: Value,
    #[serde(default)]
    pub editable: bool,
    #[serde(rename = "avatarMeta", default)]
    pub avatar_meta: String,
    #[serde(rename = "avatarPendant", default)]
    pub avatar_pendant: String,
    #[serde(rename = "feedId", default)]
    pub feed_id: String,
    #[serde(rename = "roleId", default)]
    pub role_id: String,
    #[serde(rename = "timeStamp", default)]
    pub time_stamp: String,
    #[serde(rename = "clientIdentityInfo", default)]
    pub client_identity_info: Value,
    #[serde(rename = "isImportMsg", default)]
    pub is_import_msg: bool,
    #[serde(rename = "atType", default)]
    pub at_type: i64,
    #[serde(rename = "roleType", default)]
    pub role_type: i64,
    #[serde(rename = "fromChannelRoleInfo", default)]
    pub from_channel_role_info: FromChannelRoleInfo,
    #[serde(rename = "fromGuildRoleInfo", default)]
    pub from_guild_role_info: FromGuildRoleInfo,
    #[serde(rename = "levelRoleInfo", default)]
    pub level_role_info: LevelRoleInfo,
    #[serde(rename = "recallTime", default)]
    pub recall_time: String,
    #[serde(rename = "isOnlineMsg", default)]
    pub is_online_msg: bool,
    #[serde(rename = "generalFlags", default)]
    pub general_flags: GeneralFlags,
    #[serde(rename = "clientSeq", default)]
    pub client_seq: String,
    #[serde(rename = "fileGroupSize", default)]
    pub file_group_size: Value,
    #[serde(rename = "foldingInfo", default)]
    pub folding_info: Value,
    #[serde(rename = "multiTransInfo", default)]
    pub multi_trans_info: Value,
    #[serde(rename = "senderUin", default)]
    pub sender_uin: String,
    #[serde(rename = "peerUin", default)]
    pub peer_uin: String,
    #[serde(rename = "msgAttrs", default)]
    pub msg_attrs: MsgAttrs,
    #[serde(rename = "anonymousExtInfo", default)]
    pub anonymous_ext_info: Value,
    #[serde(rename = "nameType", default)]
    pub name_type: i64,
    #[serde(rename = "avatarFlag", default)]
    pub avatar_flag: i64,
    #[serde(rename = "extInfoForUI", default)]
    pub ext_info_for_ui: Value,
    #[serde(rename = "personalMedal", default)]
    pub personal_medal: Value,
    #[serde(rename = "categoryManage", default)]
    pub category_manage: i64,
    #[serde(rename = "msgEventInfo", default)]
    pub msg_event_info: Value,
    #[serde(rename = "sourceType", default)]
    pub source_type: i64,
    #[serde(default)]
    pub id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_napcat_response() {
        let raw = r#"{
            "self_id": 1,
            "user_id": 2,
            "group_id": 100,
            "time": 1234567890,
            "message_id": 999,
            "message_seq": 888,
            "real_id": 777,
            "real_seq": "real_seq_123",
            "message_type": "group",
            "sender": {
                "user_id": 10,
                "nickname": "Cat",
                "card": "VIP"
            },
            "raw_message": "raw message",
            "font": 0,
            "sub_type": "normal",
            "message": [
                {
                    "type": "text",
                    "data": { "text": "Hello" }
                }
            ],
            "message_format": "string",
            "post_type": "message",
            "target_id": 200,
            "raw": {
                "msgId": "msg123",
                "msgRandom": "random123",
                "msgSeq": "seq123",
                "cntSeq": "cnt123",
                "chatType": 1,
                "msgType": 1,
                "subMsgType": 0,
                "sendType": 1,
                "senderUid": "uid123",
                "peerUid": "peer123",
                "channelId": "channel123",
                "guildId": "guild123",
                "guildCode": "code123",
                "fromUid": "from123",
                "fromAppid": "app123",
                "msgTime": "2023-01-01",
                "msgMeta": {},
                "sendStatus": 1,
                "sendRemarkName": "remark",
                "sendMemberName": "member",
                "sendNickName": "nick",
                "guildName": "guild",
                "channelName": "channel",
                "elements": [],
                "records": [],
                "emojiLikesList": [],
                "commentCnt": "0",
                "directMsgFlag": 0,
                "directMsgMembers": [],
                "peerName": "peer",
                "freqLimitInfo": null,
                "editable": true,
                "avatarMeta": "meta",
                "avatarPendant": "pendant",
                "feedId": "feed123",
                "roleId": "role123",
                "timeStamp": "1234567890",
                "clientIdentityInfo": null,
                "isImportMsg": false,
                "atType": 0,
                "roleType": 0,
                "fromChannelRoleInfo": {
                    "roleId": "role1",
                    "name": "role1",
                    "color": 1
                },
                "fromGuildRoleInfo": {
                    "roleId": "role2",
                    "name": "role2",
                    "color": 2
                },
                "levelRoleInfo": {
                    "roleId": "role3",
                    "name": "role3",
                    "color": 3
                },
                "recallTime": "0",
                "isOnlineMsg": true,
                "generalFlags": {},
                "clientSeq": "seq123",
                "fileGroupSize": null,
                "foldingInfo": null,
                "multiTransInfo": null,
                "senderUin": "uin123",
                "peerUin": "peeruin123",
                "msgAttrs": {},
                "anonymousExtInfo": null,
                "nameType": 0,
                "avatarFlag": 0,
                "extInfoForUI": null,
                "personalMedal": null,
                "categoryManage": 0,
                "msgEventInfo": null,
                "sourceType": 0,
                "id": 123
            }
        }"#;

        let resp: NapCatResponse = serde_json::from_str(raw).unwrap();

        assert_eq!(resp.self_id, 1);
        assert_eq!(resp.user_id, 2);
        assert_eq!(resp.group_id, 100);
        assert_eq!(resp.sender.nickname, "Cat");
        assert_eq!(resp.message[0].msg_type, "text");
        assert_eq!(resp.message[0].data.text, "Hello");
    }
}
