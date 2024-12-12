// TODO: Rather than supplying a RenderTarget that users can place into their cameras, supply
// a RatatuiCamera component that users spawn in their camera entities, that will impl From<World>
// and when initialized will create all of the image_copier/image_copier_sobel/headless_render_pipe
// infra necessary to generate the right textures and widgets.
//
// A ViewNode can then be utilized for the sobel filtering and copying instead of the
// ImageCopierList, because the ViewTarget will be filterable by the RatatuiCamera component.
