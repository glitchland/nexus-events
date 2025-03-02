/// Macro for defining event types with appropriate derives and structure.
///
/// This macro provides three patterns for defining events:
///
/// 1. Simple events with no payload
/// 2. Events with named fields (struct-style)
/// 3. Events with unnamed fields (tuple-style)
///
/// All event types automatically derive `Debug` and `Clone` as required by the `Event` trait.
///
/// # Examples
///
/// ## Simple event with no payload:
/// ```
/// use nexus_events::define_event;
///
/// // Define a simple notification event with no data
/// define_event!(GameStarted);
/// 
/// // Use it
/// let event = GameStarted;
/// ```
///
/// ## Event with named fields:
/// ```
/// use nexus_events::define_event;
///
/// // Define an event with named fields
/// define_event!(PlayerMoved {
///     player_id: String,
///     x: f32,
///     y: f32,
///     speed: f32
/// });
/// 
/// // Use it
/// let event = PlayerMoved {
///     player_id: "player1".to_string(),
///     x: 10.5,
///     y: 20.0,
///     speed: 1.5
/// };
/// ```
///
/// ## Event with unnamed fields (tuple struct):
/// ```
/// use nexus_events::define_event;
///
/// // Define a tuple-style event
/// define_event!(ScoreChanged(String, i32));
/// 
/// // Use it
/// let event = ScoreChanged("player1".to_string(), 100);
/// ```
#[macro_export]
macro_rules! define_event {
    // Simple event with no payload
    ($name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name;
    };
    
    // Event with named fields
    ($name:ident { $($field:ident: $type:ty),+ $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $type),+
        }
    };
    
    // Event with unnamed fields (tuple struct)
    ($name:ident($($type:ty),+ $(,)?)) => {
        #[derive(Debug, Clone)]
        pub struct $name($(pub $type),+);
    };
}