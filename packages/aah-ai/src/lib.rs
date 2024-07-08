pub mod utils;

use rten::Model;

const MODEL_DEPLOY_DIRECTION_CLS: &[u8; 18898316] =
    include_bytes!("../models/deploy_direction_cls.rten");
const MODEL_OPERATORS_DET: &[u8; 12246740] = include_bytes!("../models/operators_det.rten");
const MODEL_SKILL_READY_CLS: &[u8; 462448] = include_bytes!("../models/skill_ready_cls.rten");

#[cfg(test)]
mod test {
    use rten::{Dimension, FloatOperators, Model, Operators};
    use rten_tensor::{AsView, NdTensor};

    use crate::{
        utils::{image_to_tensor, ChannelOrder, DimOrder},
        MODEL_DEPLOY_DIRECTION_CLS,
    };

    #[test]
    fn foo() {
        let model = Model::load(MODEL_DEPLOY_DIRECTION_CLS.to_vec()).unwrap();
        let input_id = model.input_ids().first().copied().unwrap();
        let input_shape = model
            .node_info(input_id)
            .and_then(|info| info.shape())
            .unwrap();
        let size = 96;
        assert_eq!(
            input_shape,
            [
                Dimension::Fixed(1),
                Dimension::Fixed(3),
                Dimension::Fixed(size),
                Dimension::Fixed(size)
            ]
        );

        let image = image::open("./assets/battle0.png").unwrap();
        let image_tensor = image_to_tensor(
            image,
            ChannelOrder::Rgb,
            DimOrder::Nhwc,
            size as u32,
            size as u32,
        )
        .unwrap();

        let logits: NdTensor<f32, 2> = model
            .run_one(image_tensor.view().into(), None)
            .unwrap()
            .try_into()
            .unwrap();

        let (top_probs, top_classes) = logits
            .softmax(-1)
            .unwrap()
            .topk(5, None, true /* largest */, true /* sorted */)
            .unwrap();

        println!("Top classes:");
        for (&cls, &score) in top_classes.iter().zip(top_probs.iter()) {
            println!("{} ({})", cls, score);
        }
    }
}
