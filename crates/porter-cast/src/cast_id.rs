/// The cast node type id.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastId {
    Root = 0x746F6F72,
    Model = 0x6C646F6D,
    Mesh = 0x6873656D,
    Hair = 0x72696168,
    Skeleton = 0x6C656B73,
    Bone = 0x656E6F62,
    Animation = 0x6D696E61,
    Curve = 0x76727563,
    CurveModeOverride = 0x564F4D43,
    NotificationTrack = 0x6669746E,
    Material = 0x6C74616D,
    File = 0x656C6966,
    Color = 0x726C6F63,
    BlendShape = 0x68736C62,
    IKHandle = 0x64686B69,
    Constraint = 0x74736E63,
    Instance = 0x74736E69,
    Metadata = 0x6174656D,
}

/// The cast property type id.
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum CastPropertyId {
    Byte = b'b' as u16,
    Short = b'h' as u16,
    Integer32 = b'i' as u16,
    Integer64 = b'l' as u16,
    Float = b'f' as u16,
    Double = b'd' as u16,
    String = b's' as u16,
    Vector2 = u16::from_be_bytes(*b"v2"),
    Vector3 = u16::from_be_bytes(*b"v3"),
    Vector4 = u16::from_be_bytes(*b"v4"),
    Unknown = 0,
}
