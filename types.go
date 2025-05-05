package main

type NapCatResponse struct {
	SelfID        int       `json:"self_id"`
	UserID        int       `json:"user_id"`
	GroupID       int       `json:"group_id"`
	Time          int       `json:"time"`
	MessageID     int       `json:"message_id"`
	MessageSeq    int       `json:"message_seq"`
	RealID        int       `json:"real_id"`
	RealSeq       string    `json:"real_seq"`
	MessageType   string    `json:"message_type"`
	Sender        Sender    `json:"sender"`
	RawMessage    string    `json:"raw_message"`
	Font          int       `json:"font"`
	SubType       string    `json:"sub_type"`
	Message       []Message `json:"message"`
	MessageFormat string    `json:"message_format"`
	PostType      string    `json:"post_type"`
	TargetID      int       `json:"target_id"`
	Raw           Raw       `json:"raw"`
}
type Sender struct {
	UserID   int    `json:"user_id"`
	Nickname string `json:"nickname"`
	Card     string `json:"card"`
}
type Data struct {
	Text string `json:"text"`
}
type Message struct {
	Type string `json:"type"`
	Data Data   `json:"data"`
}
type MsgMeta struct {
}
type ExtBufForUI struct {
}
type TextElement struct {
	Content        string      `json:"content"`
	AtType         int         `json:"atType"`
	AtUID          string      `json:"atUid"`
	AtTinyID       string      `json:"atTinyId"`
	AtNtUID        string      `json:"atNtUid"`
	SubElementType int         `json:"subElementType"`
	AtChannelID    string      `json:"atChannelId"`
	LinkInfo       interface{} `json:"linkInfo"`
	AtRoleID       string      `json:"atRoleId"`
	AtRoleColor    int         `json:"atRoleColor"`
	AtRoleName     string      `json:"atRoleName"`
	NeedNotify     int         `json:"needNotify"`
}
type Elements struct {
	ElementType            int         `json:"elementType"`
	ElementID              string      `json:"elementId"`
	ElementGroupID         int         `json:"elementGroupId"`
	ExtBufForUI            ExtBufForUI `json:"extBufForUI"`
	TextElement            TextElement `json:"textElement"`
	FaceElement            interface{} `json:"faceElement"`
	MarketFaceElement      interface{} `json:"marketFaceElement"`
	ReplyElement           interface{} `json:"replyElement"`
	PicElement             interface{} `json:"picElement"`
	PttElement             interface{} `json:"pttElement"`
	VideoElement           interface{} `json:"videoElement"`
	GrayTipElement         interface{} `json:"grayTipElement"`
	ArkElement             interface{} `json:"arkElement"`
	FileElement            interface{} `json:"fileElement"`
	LiveGiftElement        interface{} `json:"liveGiftElement"`
	MarkdownElement        interface{} `json:"markdownElement"`
	StructLongMsgElement   interface{} `json:"structLongMsgElement"`
	MultiForwardMsgElement interface{} `json:"multiForwardMsgElement"`
	GiphyElement           interface{} `json:"giphyElement"`
	WalletElement          interface{} `json:"walletElement"`
	InlineKeyboardElement  interface{} `json:"inlineKeyboardElement"`
	TextGiftElement        interface{} `json:"textGiftElement"`
	CalendarElement        interface{} `json:"calendarElement"`
	YoloGameResultElement  interface{} `json:"yoloGameResultElement"`
	AvRecordElement        interface{} `json:"avRecordElement"`
	StructMsgElement       interface{} `json:"structMsgElement"`
	FaceBubbleElement      interface{} `json:"faceBubbleElement"`
	ShareLocationElement   interface{} `json:"shareLocationElement"`
	TofuRecordElement      interface{} `json:"tofuRecordElement"`
	TaskTopMsgElement      interface{} `json:"taskTopMsgElement"`
	RecommendedMsgElement  interface{} `json:"recommendedMsgElement"`
	ActionBarElement       interface{} `json:"actionBarElement"`
	PrologueMsgElement     interface{} `json:"prologueMsgElement"`
	ForwardMsgElement      interface{} `json:"forwardMsgElement"`
}
type FromChannelRoleInfo struct {
	RoleID string `json:"roleId"`
	Name   string `json:"name"`
	Color  int    `json:"color"`
}
type FromGuildRoleInfo struct {
	RoleID string `json:"roleId"`
	Name   string `json:"name"`
	Color  int    `json:"color"`
}
type LevelRoleInfo struct {
	RoleID string `json:"roleId"`
	Name   string `json:"name"`
	Color  int    `json:"color"`
}
type GeneralFlags struct {
}
type MsgAttrs struct {
}
type Raw struct {
	MsgID               string              `json:"msgId"`
	MsgRandom           string              `json:"msgRandom"`
	MsgSeq              string              `json:"msgSeq"`
	CntSeq              string              `json:"cntSeq"`
	ChatType            int                 `json:"chatType"`
	MsgType             int                 `json:"msgType"`
	SubMsgType          int                 `json:"subMsgType"`
	SendType            int                 `json:"sendType"`
	SenderUID           string              `json:"senderUid"`
	PeerUID             string              `json:"peerUid"`
	ChannelID           string              `json:"channelId"`
	GuildID             string              `json:"guildId"`
	GuildCode           string              `json:"guildCode"`
	FromUID             string              `json:"fromUid"`
	FromAppid           string              `json:"fromAppid"`
	MsgTime             string              `json:"msgTime"`
	MsgMeta             MsgMeta             `json:"msgMeta"`
	SendStatus          int                 `json:"sendStatus"`
	SendRemarkName      string              `json:"sendRemarkName"`
	SendMemberName      string              `json:"sendMemberName"`
	SendNickName        string              `json:"sendNickName"`
	GuildName           string              `json:"guildName"`
	ChannelName         string              `json:"channelName"`
	Elements            []Elements          `json:"elements"`
	Records             []interface{}       `json:"records"`
	EmojiLikesList      []interface{}       `json:"emojiLikesList"`
	CommentCnt          string              `json:"commentCnt"`
	DirectMsgFlag       int                 `json:"directMsgFlag"`
	DirectMsgMembers    []interface{}       `json:"directMsgMembers"`
	PeerName            string              `json:"peerName"`
	FreqLimitInfo       interface{}         `json:"freqLimitInfo"`
	Editable            bool                `json:"editable"`
	AvatarMeta          string              `json:"avatarMeta"`
	AvatarPendant       string              `json:"avatarPendant"`
	FeedID              string              `json:"feedId"`
	RoleID              string              `json:"roleId"`
	TimeStamp           string              `json:"timeStamp"`
	ClientIdentityInfo  interface{}         `json:"clientIdentityInfo"`
	IsImportMsg         bool                `json:"isImportMsg"`
	AtType              int                 `json:"atType"`
	RoleType            int                 `json:"roleType"`
	FromChannelRoleInfo FromChannelRoleInfo `json:"fromChannelRoleInfo"`
	FromGuildRoleInfo   FromGuildRoleInfo   `json:"fromGuildRoleInfo"`
	LevelRoleInfo       LevelRoleInfo       `json:"levelRoleInfo"`
	RecallTime          string              `json:"recallTime"`
	IsOnlineMsg         bool                `json:"isOnlineMsg"`
	GeneralFlags        GeneralFlags        `json:"generalFlags"`
	ClientSeq           string              `json:"clientSeq"`
	FileGroupSize       interface{}         `json:"fileGroupSize"`
	FoldingInfo         interface{}         `json:"foldingInfo"`
	MultiTransInfo      interface{}         `json:"multiTransInfo"`
	SenderUin           string              `json:"senderUin"`
	PeerUin             string              `json:"peerUin"`
	MsgAttrs            MsgAttrs            `json:"msgAttrs"`
	AnonymousExtInfo    interface{}         `json:"anonymousExtInfo"`
	NameType            int                 `json:"nameType"`
	AvatarFlag          int                 `json:"avatarFlag"`
	ExtInfoForUI        interface{}         `json:"extInfoForUI"`
	PersonalMedal       interface{}         `json:"personalMedal"`
	CategoryManage      int                 `json:"categoryManage"`
	MsgEventInfo        interface{}         `json:"msgEventInfo"`
	SourceType          int                 `json:"sourceType"`
	ID                  int                 `json:"id"`
}
