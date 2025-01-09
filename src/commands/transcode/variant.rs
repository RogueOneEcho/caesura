use crate::commands::*;

pub enum Variant {
    Transcode(Decode, Encode),
    Resample(Resample),
}
