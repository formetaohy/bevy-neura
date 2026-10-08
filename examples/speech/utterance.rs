use bevy::prelude::*;

#[derive(Message)]
pub struct Utterance {
    pub text: String,
}
