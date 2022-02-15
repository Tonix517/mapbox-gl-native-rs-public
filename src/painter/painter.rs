use gfx;
use gfx::format::{Srgb, R8_G8_B8_A8};
use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_device_gl::Factory;
use gfx_text;
use gfx_window_glutin as gfx_glutin;
use glutin::dpi::LogicalSize;
use glutin::Api::OpenGl;
use glutin::{EventsLoop, GlRequest, PossiblyCurrent};

use super::bucket::*;
use super::tile_grid;

use crate::config;
use crate::mapbox::common::types::{Threadable, ThreadableNew};
use crate::mapbox::map::Map;
use crate::mapbox::vector_tile_id::VectorTileID;
use crate::mapbox::vector_tile_model::VectorTileModel;

use crate::mapbox::vector_tile_observer::VectorTileObserver;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const SEA: [f32; 4] = [0.6745, 0.8352, 0.9921, 1.0];

struct GfxDelegate {
    pub device: gfx_device_gl::Device,
    pub window: glutin::WindowedContext<PossiblyCurrent>,
    pub factory: Factory,
    pub encoder: gfx::Encoder<ResourceType, gfx_device_gl::CommandBuffer>,
    pub color_view: gfx::handle::RenderTargetView<ResourceType, (R8_G8_B8_A8, Srgb)>,

    pub pso: gfx::PipelineState<ResourceType, pipe::Meta>,
    pub grid_pso: gfx::PipelineState<ResourceType, pipe::Meta>,
    pub linestrip_pso: gfx::PipelineState<ResourceType, pipe::Meta>,
    pub polygon_pso: gfx::PipelineState<ResourceType, pipe::Meta>,
    pub points_pso: gfx::PipelineState<ResourceType, pipe::Meta>,
}

impl GfxDelegate {
    pub fn new(events_loop: &EventsLoop) -> Self {
        let window_builder = glutin::WindowBuilder::new()
            .with_title("Tony's Mapbox Renderer".to_string())
            .with_dimensions(LogicalSize {
                width: config::GL_VIEWPORT_WIDTH as f64,
                height: config::GL_VIEWPORT_HEIGHT as f64,
            });
        let context_builder = glutin::ContextBuilder::new()
            .with_gl(GlRequest::Specific(OpenGl, (3, 2)))
            .with_vsync(true);

        let (window, device, mut factory, color_view, mut _depth_view) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(
                window_builder,
                context_builder,
                &events_loop,
            )
            .unwrap();

        let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let mut fillmode = gfx::state::Rasterizer::new_fill();
        fillmode.method = gfx::state::RasterMethod::Fill;
        fillmode.front_face = gfx::state::FrontFace::Clockwise;
        let vs = include_bytes!("shaders/simple.glslv");
        let fs = include_bytes!("shaders/simple.glslf");
        let shader_set = factory.create_shader_set(vs, fs).unwrap();
        let pso = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::TriangleList,
                fillmode,
                pipe::new(),
            )
            .unwrap();

        let mut grid_fillmode = gfx::state::Rasterizer::new_fill();
        grid_fillmode.method = gfx::state::RasterMethod::Line(1);
        let grid_pso = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::LineStrip,
                grid_fillmode,
                pipe::new(),
            )
            .unwrap();

        let linestrip_pso = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::LineStrip,
                grid_fillmode,
                pipe::new(),
            )
            .unwrap();

        let polygon_pso = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::LineStrip,
                fillmode,
                pipe::new(),
            )
            .unwrap();

        let mut point_fillmode = gfx::state::Rasterizer::new_fill();
        point_fillmode.method = gfx::state::RasterMethod::Point;
        let points_pso = factory
            .create_pipeline_state(
                &shader_set,
                gfx::Primitive::PointList,
                point_fillmode,
                pipe::new(),
            )
            .unwrap();

        GfxDelegate {
            device,
            window,
            factory,
            encoder,
            color_view,
            pso,
            grid_pso,
            linestrip_pso,
            polygon_pso,
            points_pso,
        }
    }
}

pub struct Painter {
    map: Arc<RefCell<Map>>,
    gfx_delegate: GfxDelegate,
    // Markers
    need_update: bool,
    show_tile_grid: bool,
    //
    painter_impl: Threadable<PainterImpl>,
    //
    bucket: Threadable<Bucket>,
    //
    text: gfx_text::Renderer<ResourceType, Factory>,
    //
    tile_screen_coords: Threadable<HashMap<VectorTileID, (f32, f32, f32, f32)>>,
}

impl Painter {
    pub fn new(map: Arc<RefCell<Map>>, events_loop: &EventsLoop) -> Self {
        let gfx_delegate = GfxDelegate::new(events_loop);
        let bucket = ThreadableNew(Bucket::new());

        // gfx_text::new() is slow enough to move it out of drawcall.
        let text = gfx_text::new(gfx_delegate.factory.clone())
            .with_size(32)
            .unwrap();

        let tile_screen_coords = ThreadableNew(HashMap::new());

        let mut painter = Painter {
            map,
            gfx_delegate,
            need_update: true,
            show_tile_grid: true,
            painter_impl: ThreadableNew(PainterImpl::new(bucket.clone())),
            bucket,
            text,
            tile_screen_coords,
        };

        // TODO: support panning
        painter.calc_tile_screenspace_rect();

        painter
            .map
            .borrow_mut()
            .add_vector_tile_observer(painter.painter_impl.clone());

        painter
    }

    fn calc_tile_screenspace_rect(&mut self) {
        let covered_tilex_xy = self.map.borrow().get_covered_tiles_coords();

        for tile_coord in covered_tilex_xy {
            let (center_tile_x, center_tile_y) = self.map.borrow().get_center_point_tile_xy();
            let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) =
                tile_grid::TileGrid::gen_tile_grid_screen_coords(
                    (center_tile_x, center_tile_y),
                    (tile_coord.0, tile_coord.1),
                );
            self.tile_screen_coords.lock().unwrap().insert(
                VectorTileID {
                    x: tile_coord.0 as u32,
                    y: tile_coord.1 as u32,
                    z: self.map.borrow().get_zoom() as u32,
                },
                (top_left_x, top_left_y, bottom_right_x, bottom_right_y),
            );
        }

        self.painter_impl
            .lock()
            .unwrap()
            .set_tile_screen_coords(self.tile_screen_coords.clone());
    }

    fn get_tile_screenspace_rect(&self, vector_tile_id: VectorTileID) -> (f32, f32, f32, f32) {
        let tile_screen_coords = self.tile_screen_coords.lock().unwrap();
        tile_screen_coords.get(&vector_tile_id).unwrap().clone()
    }

    pub fn set_need_update(&mut self, need_update: bool) {
        self.need_update = need_update;
    }

    pub fn toggle_show_grid(&mut self) {
        self.show_tile_grid = !self.show_tile_grid;
        println!("= Show Grid: {}", self.show_tile_grid);
    }

    fn vertex_array_to_data(
        &mut self,
        vertices: &Vec<VertexType>,
    ) -> (pipe::Data<ResourceType>, gfx::Slice<ResourceType>) {
        let (vertex_buffer, slice) = self
            .gfx_delegate
            .factory
            .create_vertex_buffer_with_slice(vertices, {});

        let transform_buffer = self.gfx_delegate.factory.create_constant_buffer(1);
        let data = pipe::Data {
            vbuf: vertex_buffer,
            transform: transform_buffer,
            out: self.gfx_delegate.color_view.clone(),
        };

        (data, slice)
    }

    fn tuple_vec_to_vertex_array(
        &self,
        top_left_screen_coords: (f32, f32),
        tile_scale_factor: (f32, f32),
        geom: &Vec<(f32, f32)>,
    ) -> Vec<VertexType> {
        let mut arr: Vec<VertexType> = vec![];

        for point in geom.iter() {
            arr.push(VertexType {
                pos: [
                    top_left_screen_coords.0 + point.0 * tile_scale_factor.0,
                    top_left_screen_coords.1 - point.1 * tile_scale_factor.1,
                    0.0,
                    1.0,
                ],
                color: [0.0, 0.0, 1.0, 1.0], // TODO
            });
        }

        arr
    }

    fn gen_data(
        &mut self,
        top_left_screen_coords: (f32, f32),
        tile_scale_factor: (f32, f32),
        geom: &Vec<(f32, f32)>,
    ) -> (pipe::Data<ResourceType>, gfx::Slice<ResourceType>) {
        let arr = self.tuple_vec_to_vertex_array(top_left_screen_coords, tile_scale_factor, geom);
        self.vertex_array_to_data(&arr)
    }

    fn gen_grid_data(
        &mut self,
        target_tile_coords: (f32, f32),
    ) -> (
        pipe::Data<ResourceType>,
        gfx::Slice<ResourceType>,
        (f32, f32),
    ) {
        let vector_tile_id = VectorTileID {
            x: target_tile_coords.0 as u32,
            y: target_tile_coords.1 as u32,
            z: self.map.borrow().get_zoom() as u32,
        };
        let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) =
            self.get_tile_screenspace_rect(vector_tile_id);

        let lines: [VertexType; 5] = [
            VertexType {
                pos: [top_left_x, top_left_y, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            VertexType {
                pos: [bottom_right_x, top_left_y, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            VertexType {
                pos: [bottom_right_x, bottom_right_y, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            VertexType {
                pos: [top_left_x, bottom_right_y, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            VertexType {
                pos: [top_left_x, top_left_y, 0.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        let (vertex_buffer, slice) = self
            .gfx_delegate
            .factory
            .create_vertex_buffer_with_slice(&lines, {});

        let transform_buffer = self.gfx_delegate.factory.create_constant_buffer(1);
        let data = pipe::Data {
            vbuf: vertex_buffer,
            transform: transform_buffer,
            out: self.gfx_delegate.color_view.clone(),
        };

        (data, slice, (top_left_x, top_left_y))
    }

    fn render_tile_grid(&mut self) {
        let covered_tilex_xy = self.map.borrow().get_covered_tiles_coords();

        // TODO: we can definitely do smarter line gen and rendering here.
        for tile_coord in covered_tilex_xy {
            let (data, slice, top_left_coord) = self.gen_grid_data(tile_coord);

            //Identity Matrix
            const TRANSFORM: TransformType = TransformType {
                screen_ratio: config::GL_VIEWPORT_WIDTH as f32 / config::GL_VIEWPORT_HEIGHT as f32,
            };
            self.gfx_delegate
                .encoder
                .update_buffer(&data.transform, &[TRANSFORM], 0)
                .unwrap_or_default();

            self.gfx_delegate
                .encoder
                .draw(&slice, &self.gfx_delegate.grid_pso, &data);

            // Render Text
            let (text_x, text_y) = (
                config::GL_VIEWPORT_WIDTH as f32 * (1.0 + top_left_coord.0),
                config::GL_VIEWPORT_HEIGHT as f32 * (1.0 - top_left_coord.1),
            );

            let tile_coord_text = format!(
                "{}/{}/{}",
                self.map.borrow().get_zoom(),
                tile_coord.0 as u32,
                tile_coord.1 as u32,
            );
            // TODO: why gfx_text::Renderer doesn't support clear()?
            self.text.add(
                &tile_coord_text,
                [text_x as i32 + 5, text_y as i32 + 5],
                [1.0, 0.0, 0.0, 1.0],
            );

            self.text
                .draw(
                    &mut self.gfx_delegate.encoder,
                    &self.gfx_delegate.color_view,
                )
                .unwrap_or_default();
        }
    }

    fn render_vector_tile(&mut self, vector_tile_name: String) {
        let vector_tiles_map = self
            .bucket
            .lock()
            .unwrap()
            .vector_tiles_map
            .lock()
            .unwrap()
            .clone();
        let loaded_vector_tiles = vector_tiles_map.get(&vector_tile_name).unwrap();

        let covered_tilex_xy = self.map.borrow().get_covered_tiles_coords();
        for tile_coord in covered_tilex_xy {
            let vector_tile_id = VectorTileID {
                x: tile_coord.0 as u32,
                y: tile_coord.1 as u32,
                z: self.map.borrow().get_zoom() as u32,
            };

            if !loaded_vector_tiles.contains_key(&vector_tile_id) {
                continue;
            }

            let (top_left_x, top_left_y, _bottom_right_x, _bottom_right_y) =
                self.get_tile_screenspace_rect(vector_tile_id);

            // Render
            let vector_tile_model = loaded_vector_tiles.get(&vector_tile_id).unwrap();

            let renderable = self
                .bucket
                .lock()
                .unwrap()
                .is_vector_tile_renderable(&vector_tile_name, vector_tile_id.clone());
            if !renderable {
                // TODO: MAJOR: merge point and line data into one drawcall.

                let tile_dim_screen_ratio =
                    tile_grid::TileGrid::get_normalized_tile_dim_screen_ratio();

                let mut point_vertices: Vec<VertexType> = vec![];
                let mut linestrip_vertices: Vec<VertexType> = vec![];

                for layer in vector_tile_model.layers.iter() {
                    for feature in layer.features.iter() {
                        let geom_type = feature.r#type;
                        // 1: points
                        // 2: lines
                        // 3: polygon
                        // TODO: WATER polygons is confusing - will need to avoid it as a tmp solution.
                        for geom_set in feature.geometry.iter() {
                            for geom in geom_set.geom_set.iter() {
                                match geom_type {
                                    3 => {
                                        // TODO: we can definitely apply the same merging trick to polygons to put
                                        //       all polygons' render into just one drawcall. It can be possible
                                        //       after all these polygons are triangulated and then we render it
                                        //       in TriangleList mode.
                                        // Polygon
                                        let (render_data, render_slice) = self.gen_data(
                                            (top_left_x, top_left_y),
                                            tile_dim_screen_ratio,
                                            geom,
                                        );

                                        self.bucket.lock().unwrap().add_renderable_item(
                                            vector_tile_name.clone(),
                                            vector_tile_id,
                                            RenderableItem {
                                                geometry_type: geom_type,
                                                data: render_data.clone(),
                                                slice: render_slice.clone(),
                                            },
                                        );
                                    }
                                    1 => {
                                        // points
                                        let mut points = self.tuple_vec_to_vertex_array(
                                            (top_left_x, top_left_y),
                                            tile_dim_screen_ratio,
                                            geom,
                                        );
                                        point_vertices.append(&mut points);
                                    }
                                    2 => {
                                        // lines
                                        // Ah here is the trick
                                        let mut lines = self.tuple_vec_to_vertex_array(
                                            (top_left_x, top_left_y),
                                            tile_dim_screen_ratio,
                                            geom,
                                        );

                                        if !linestrip_vertices.is_empty() && !lines.is_empty() {
                                            let mut last =
                                                linestrip_vertices.last().unwrap().clone();
                                            let mut next_first = lines.first().unwrap().clone();
                                            last.color[3] = 0.0;
                                            next_first.color[3] = 0.0;
                                            linestrip_vertices.push(last);
                                            linestrip_vertices.push(next_first);
                                        }

                                        linestrip_vertices.append(&mut lines);
                                    }
                                    _ => {
                                        // not supported
                                    }
                                }
                            }
                        } // geom_set
                    } // feature
                } // layer

                // Add merged points
                if !point_vertices.is_empty() {
                    let (render_data, render_slice) = self.vertex_array_to_data(&point_vertices);
                    self.bucket.lock().unwrap().add_renderable_item(
                        vector_tile_name.clone(),
                        vector_tile_id,
                        RenderableItem {
                            geometry_type: 1,
                            data: render_data.clone(),
                            slice: render_slice.clone(),
                        },
                    );
                }

                // Add merged lines
                if !linestrip_vertices.is_empty() {
                    let (render_data, render_slice) =
                        self.vertex_array_to_data(&linestrip_vertices);
                    self.bucket.lock().unwrap().add_renderable_item(
                        vector_tile_name.clone(),
                        vector_tile_id,
                        RenderableItem {
                            geometry_type: 2,
                            data: render_data.clone(),
                            slice: render_slice.clone(),
                        },
                    );
                }
            } // if !renderable

            let renderable_items = self
                .bucket
                .lock()
                .unwrap()
                .get_renderable_items(vector_tile_name.clone(), vector_tile_id.clone());
            for renderable_item in renderable_items.lock().unwrap().iter() {
                //Identity Matrix
                const TRANSFORM: TransformType = TransformType {
                    screen_ratio: config::GL_VIEWPORT_WIDTH as f32
                        / config::GL_VIEWPORT_HEIGHT as f32,
                };
                self.gfx_delegate
                    .encoder
                    .update_buffer(&renderable_item.data.transform, &[TRANSFORM], 0)
                    .unwrap_or_default();

                match renderable_item.geometry_type {
                    1 => {
                        self.gfx_delegate.encoder.draw(
                            &renderable_item.slice,
                            &self.gfx_delegate.points_pso,
                            &renderable_item.data,
                        );
                    }
                    2 => {
                        self.gfx_delegate.encoder.draw(
                            &renderable_item.slice,
                            &self.gfx_delegate.linestrip_pso,
                            &renderable_item.data,
                        );
                    }
                    3 => {
                        self.gfx_delegate.encoder.draw(
                            &renderable_item.slice,
                            &self.gfx_delegate.polygon_pso,
                            &renderable_item.data,
                        );
                    }
                    _ => {
                        // TODO: not supported
                    }
                }
            } // for renderables
        }

        self.painter_impl.lock().unwrap().set_dirty(false);
    }

    pub fn render(&mut self) {
        if !self.need_update && !self.painter_impl.lock().unwrap().is_dirty() {
            return;
        }
        self.gfx_delegate
            .encoder
            .clear(&self.gfx_delegate.color_view, SEA);

        // Render Map Data
        self.render_vector_tile("COMPOSITE".to_string());
        self.render_vector_tile("BUILDINGS".to_string());

        // Render Tile Grids
        if self.show_tile_grid {
            self.render_tile_grid();
        }

        self.gfx_delegate
            .encoder
            .flush(&mut self.gfx_delegate.device);

        self.gfx_delegate.window.swap_buffers().unwrap();
        self.gfx_delegate.device.cleanup();
    }
}

struct PainterImpl {
    dirty: AtomicBool,
    bucket: Threadable<Bucket>,
    tile_screen_coords: Threadable<HashMap<VectorTileID, (f32, f32, f32, f32)>>,
}

impl PainterImpl {
    pub fn new(bucket: Threadable<Bucket>) -> Self {
        PainterImpl {
            dirty: AtomicBool::new(true),
            bucket,
            tile_screen_coords: ThreadableNew(HashMap::new()),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty.store(dirty, Ordering::Relaxed);
    }

    pub fn set_tile_screen_coords(
        &mut self,
        tile_screen_coords: Threadable<HashMap<VectorTileID, (f32, f32, f32, f32)>>,
    ) {
        self.tile_screen_coords = tile_screen_coords;
    }
}

impl VectorTileObserver for PainterImpl {
    fn on_vector_tile_loaded(
        &mut self,
        name: String,
        vector_tile_id: VectorTileID,
        parsed_vector_tile: Arc<VectorTileModel>,
    ) {
        let mut bucket = self.bucket.lock().unwrap();
        if bucket.contains(&name, vector_tile_id.clone()) {
            println!(
                "VectorTile {:?} already loaded with name {}- reloaded",
                vector_tile_id, name
            );
            bucket.remove(&name, vector_tile_id.clone());
        }

        bucket.add_vector_tile_data(&name, vector_tile_id.clone(), parsed_vector_tile.clone());

        self.dirty.store(true, Ordering::Relaxed)
    }
}
