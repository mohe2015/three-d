use crate::core::*;

pub struct RenderTargetMultisample {
    context: Context,
    color: Option<Texture2DMultisample>,
    depth: Option<DepthTexture2DMultisample>,
}

impl RenderTargetMultisample {
    pub fn new_color<T: TextureDataType>(
        context: &Context,
        width: u32,
        height: u32,
        number_of_samples: u32,
    ) -> Self {
        let color = Texture2DMultisample::new::<T>(context, width, height, number_of_samples);
        Self {
            context: context.clone(),
            color: Some(color),
            depth: None,
        }
    }

    /// The width of this target.
    pub fn width(&self) -> u32 {
        self.color
            .as_ref()
            .map(|t| t.width())
            .unwrap_or_else(|| self.depth.as_ref().unwrap().width())
    }

    /// The height of this target.
    pub fn height(&self) -> u32 {
        self.color
            .as_ref()
            .map(|t| t.height())
            .unwrap_or_else(|| self.depth.as_ref().unwrap().height())
    }

    /// The number of samples for each fragment.
    pub fn number_of_samples(&self) -> u32 {
        self.color
            .as_ref()
            .map(|t| t.number_of_samples())
            .unwrap_or_else(|| self.depth.as_ref().unwrap().number_of_samples())
    }

    pub fn as_render_target(&mut self) -> RenderTarget<'_> {
        if let Some(color) = &mut self.color {
            RenderTarget::new_color(color.as_color_target())
        } else {
            unimplemented!()
        }
    }

    pub fn resolve(&self) -> Option<Texture2D> {
        if let Some(color) = &self.color {
            let mut color_texture = Texture2D::new_empty::<[u8; 4]>(
                &self.context,
                color.width(),
                color.height(),
                Interpolation::Nearest,
                Interpolation::Nearest,
                None,
                Wrapping::ClampToEdge,
                Wrapping::ClampToEdge,
            );

            {
                let target = color_texture.as_color_target(None).as_render_target();

                let source = ColorTarget::new_texture_2d_multisample(&self.context, &color)
                    .as_render_target();

                source.blit(&target);
            }
            Some(color_texture)
        } else {
            None
        }
    }
}
