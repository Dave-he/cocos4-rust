/****************************************************************************
Rust port of Cocos Creator Audio Decoder
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Unknown = 0,
    Mp3 = 1,
    Ogg = 2,
    Wav = 3,
    Aac = 4,
    Mp4 = 5,
}

#[derive(Debug, Clone)]
pub struct AudioDecoderInfo {
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channel_count: u32,
    pub bits_per_sample: u32,
    pub total_frames: u64,
}

impl Default for AudioDecoderInfo {
    fn default() -> Self {
        AudioDecoderInfo {
            format: AudioFormat::Unknown,
            sample_rate: 44100,
            channel_count: 2,
            bits_per_sample: 16,
            total_frames: 0,
        }
    }
}

pub struct PcmAudioDecoder {
    pub info: AudioDecoderInfo,
    data: Vec<u8>,
    position: usize,
}

impl PcmAudioDecoder {
    pub fn new(data: Vec<u8>, info: AudioDecoderInfo) -> Self {
        PcmAudioDecoder {
            info,
            data,
            position: 0,
        }
    }

    pub fn read(&mut self, frames: u32) -> Vec<i16> {
        let bytes_per_frame = (self.info.bits_per_sample / 8 * self.info.channel_count) as usize;
        let byte_count = frames as usize * bytes_per_frame;
        let available = (self.data.len() - self.position).min(byte_count);

        let mut output = Vec::with_capacity(available / 2);
        for i in (0..available).step_by(2) {
            if self.position + i + 1 < self.data.len() {
                let lo = self.data[self.position + i] as i16;
                let hi = self.data[self.position + i + 1] as i16;
                output.push((hi << 8) | lo);
            }
        }
        self.position += available;
        output
    }

    pub fn seek(&mut self, frame: u64) {
        let bytes_per_frame =
            (self.info.bits_per_sample / 8 * self.info.channel_count) as usize;
        self.position = (frame as usize * bytes_per_frame).min(self.data.len());
    }

    pub fn get_current_frame(&self) -> u64 {
        let bytes_per_frame =
            (self.info.bits_per_sample / 8 * self.info.channel_count) as usize;
        if bytes_per_frame == 0 {
            0
        } else {
            (self.position / bytes_per_frame) as u64
        }
    }

    pub fn is_end(&self) -> bool {
        self.position >= self.data.len()
    }

    pub fn get_duration(&self) -> f32 {
        if self.info.sample_rate == 0 {
            0.0
        } else {
            self.info.total_frames as f32 / self.info.sample_rate as f32
        }
    }
}
