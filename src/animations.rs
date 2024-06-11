use keyframe::{functions::EaseInOut, keyframes, mint::Point2, AnimationSequence, Keyframe};

pub struct Animation {
    pub rotate_keyframes: AnimationSequence<f32>,
    pub scale_keyframes: AnimationSequence<Point2<f32>>,
    pub translate_keyframes: AnimationSequence<Point2<f32>>,
}

pub struct AnimationParams {
    pub start_x: f32,
    pub start_y: f32,
    pub start_width: f32,
    pub start_height: f32,
    pub destination_width: f32,
    pub destination_height: f32,
    pub destination_x: f32,
    pub destination_y: f32,
}

const MACOS_FUDGE_FACTOR_FOR_BAR: f32 = if cfg!(target_os = "macos") {
    -25.0
} else {
    0.0
};

impl Animation {
    pub fn boring(params: &AnimationParams) -> Animation {
        let scale_keyframes = keyframes![
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.start_width,
                    y: params.start_height,
                },
                0.0,
                EaseInOut
            ),
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.destination_width,
                    y: params.destination_height,
                },
                1.0,
                EaseInOut
            )
        ];

        let translate_keyframes = keyframes![
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.start_x,
                    y: params.start_y - MACOS_FUDGE_FACTOR_FOR_BAR,
                },
                0.0,
                EaseInOut
            ),
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.destination_x,
                    y: params.destination_y,
                },
                1.0,
                EaseInOut
            )
        ];

        let rotate_keyframes = keyframes![
            Keyframe::new(0.0, 0.0, EaseInOut),
            Keyframe::new(0.0, 1.0, EaseInOut)
        ];

        Animation {
            rotate_keyframes,
            scale_keyframes,
            translate_keyframes,
        }
    }

    pub fn spin_move(params: &AnimationParams) -> Animation {
        let scale_keyframes = keyframes![
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.start_width,
                    y: params.start_height,
                },
                0.0,
                EaseInOut
            ),
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.destination_width,
                    y: params.destination_height,
                },
                1.0,
                EaseInOut
            )
        ];

        let translate_keyframes = keyframes![
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.start_x,
                    y: params.start_y - MACOS_FUDGE_FACTOR_FOR_BAR,
                },
                0.0,
                EaseInOut
            ),
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.destination_x,
                    y: params.destination_y,
                },
                1.0,
                EaseInOut
            )
        ];

        let rotate_keyframes = keyframes![
            Keyframe::new(0.0, 0.0, EaseInOut),
            Keyframe::new(180.0, 1.0, EaseInOut)
        ];

        Animation {
            rotate_keyframes,
            scale_keyframes,
            translate_keyframes,
        }
    }

    pub fn spin_move2(params: &AnimationParams) -> Animation {
        let scale_keyframes = keyframes![
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.start_width,
                    y: params.start_height,
                },
                0.0,
                EaseInOut
            ),
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.destination_width,
                    y: params.destination_height,
                },
                1.0,
                EaseInOut
            )
        ];

        let translate_keyframes = keyframes![
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.start_x,
                    y: params.start_y - MACOS_FUDGE_FACTOR_FOR_BAR,
                },
                0.0,
                EaseInOut
            ),
            Keyframe::new(
                keyframe::mint::Point2 {
                    x: params.destination_x,
                    y: params.destination_y,
                },
                1.0,
                EaseInOut
            )
        ];

        let rotate_keyframes = keyframes![
            Keyframe::new(0.0, 0.0, EaseInOut),
            Keyframe::new(0.0, 0.3, EaseInOut),
            Keyframe::new(180.0, 1.0, EaseInOut)
        ];

        Animation {
            rotate_keyframes,
            scale_keyframes,
            translate_keyframes,
        }
    }
}
