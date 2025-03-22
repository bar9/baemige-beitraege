use gdal::Dataset;
use gdal::vector::{Geometry, LayerAccess};
use geo::{EuclideanDistance, Point};
use image::{DynamicImage, ImageReader};
use minifb::{Window, WindowOptions, Key, MouseButton, MouseMode};
use minifb::Key::P;

fn main() {
    let gpkg_path = "data/SWISSTLM3D_2025.gpkg";
    let layer_name = "tlm_bb_einzelbaum_gebuesch";
    let shapefile_path = "data/outline_liebegg.shp";

    // Open the shapefile and dataset
    let dataset = Dataset::open(shapefile_path).unwrap();
    let layer = dataset.layer(0).unwrap();
    println!("Opened layer: {}", layer.name());

    // Get extent of the layer
    let extent = layer.get_extent().unwrap();
    let minx = extent.MinX;
    let miny = extent.MinY;
    let maxx = extent.MaxX;
    let maxy = extent.MaxY;
    println!("Layer extent: minx: {}, miny: {}, maxx: {}, maxy: {}", minx, miny, maxx, maxy);

    // Create a bounding box geometry using the extent
    let bounding_box_geometry = Geometry::from_wkt(
        &format!("POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))", minx, miny, maxx, miny, maxx, maxy, minx, maxy, minx, miny)
    ).unwrap();

    // Open GeoPackage and layer
    match Dataset::open(gpkg_path) {
        Ok(dataset) => {
            match dataset.layer_by_name(layer_name) {
                Ok(mut layer) => {
                    println!("Opened layer: {}", layer_name);
                    layer.set_spatial_filter(&bounding_box_geometry);

                    // Extract tree geometries (simplified for example)
                    let mut trees: Vec<Tree> = layer.features().filter_map(|feature| {
                        if let Some(geometry) = feature.geometry() {
                            Some(Tree { geom: geometry.clone() })
                        } else {
                            None
                        }
                    }).collect();

                    // Initialize the window for rendering
                    let mut window = Window::new("Liebegg", 800, 300, WindowOptions {
                        resize: true,
                        ..Default::default()
                    }).unwrap();

                    let png_path = "data/background_liebegg.png"; // Path to your PNG

                    // Load the image and convert to an RGB buffer
                    let img = ImageReader::open(png_path)
                        .expect("Failed to open image")
                        .decode()
                        .expect("Failed to decode image");

                    let img = match img {
                        DynamicImage::ImageRgb8(img) => img,
                        other => other.to_rgb8(), // Convert other formats to RGB8
                    };

                    // window.update_with_buffer(&buffer, 800, 300).unwrap();

                    // Main loop for window and event handling
                    while window.is_open() && !window.is_key_down(Key::Escape) {


                        if let Some((x, mut y)) = window.get_mouse_pos(MouseMode::Clamp) {


                            // y = 300. - y;

                            let img_width = img.width() as usize;
                            let img_height = img.height() as usize;
                            let mut buffer: Vec<u32> = vec![0xFFFFFF; 800 * 300];
                            let mut mouse_down_last_frame = false;
                            let mouse_down = window.get_mouse_down(MouseButton::Left);


                            // Copy image pixels to buffer, resizing if necessary
                            for y in 0..300 {
                                for x in 0..800 {
                                    let src_x = (x * img_width) / 800;
                                    let src_y = (y * img_height) / 300;
                                    let pixel = img.get_pixel(src_x as u32, src_y as u32);
                                    let r = pixel[0] as u32;
                                    let g = pixel[1] as u32;
                                    let b = pixel[2] as u32;
                                    buffer[y * 800 + x] = (r << 16) | (g << 8) | b;
                                }
                            }

                            for tree in &trees {
                                let (map_x, map_y) = (tree.geom.get_point(0).0, tree.geom.get_point(0).1);

                                let tx = ((map_x - minx) / (maxx - minx) * 800.0) as i32;
                                let ty = ((maxy - map_y) / (maxy - miny) * 300.0) as i32;
                                if tx >= 0 && tx < 800 && ty >= 0 && ty < 300 {
                                    buffer[(ty as usize * 800 + tx as usize)] = 0x000000;
                                    buffer[((ty+1) as usize * 800 + tx as usize)] = 0x000000;
                                    buffer[((ty+2) as usize * 800 + tx as usize)] = 0x000000;
                                    buffer[(ty as usize * 800 + (tx-1) as usize)] = 0x000000;
                                    buffer[(ty as usize * 800 + (tx-2) as usize)] = 0x000000;
                                    buffer[((ty-1) as usize * 800 + tx as usize)] = 0x000000;
                                    buffer[((ty-2) as usize * 800 + tx as usize)] = 0x000000;
                                    buffer[(ty as usize * 800 + (tx+1) as usize)] = 0x000000;
                                    buffer[(ty as usize * 800 + (tx+2) as usize)] = 0x000000;
                                }
                            }

                            // Transform pixel coordinates (x, y) to spatial coordinates
                            let spatial_x = minx + (x as f64 / 800.0) * (maxx - minx);
                            let spatial_y = maxy - (y as f64 / 300.0) * (maxy - miny);

                            // println!("spatial x, y: {}/{}", spatial_x, spatial_y);

                            let click_pos = Point::new(spatial_x, spatial_y);
                            let new_tree = Tree {
                                geom: Geometry::from_wkt(&format!("POINT({} {})", spatial_x, spatial_y)).unwrap(),
                            };

                            if (mouse_down && !mouse_down_last_frame) {
                                trees.push(new_tree.clone());
                                mouse_down_last_frame = true;
                            }

                            // if mouse_down && !mouse_down_last_frame {
                            // if mouse_down {
                                buffer = heatmap_step_30m(new_tree, &trees, buffer, maxx, minx, maxy, miny);
                                // buffer = vec![0xFFFFFF; 800 * 300];
                                window.update_with_buffer(&buffer, 800, 300).unwrap();
                            // } else {
                            //     window.update_with_buffer(&buffer, 800, 300).unwrap();
                            // }

                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open layer {}: {}", layer_name, e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open GeoPackage: {}", e);
        }
    }
}

fn heatmap_step_30m(new_tree: Tree, trees: &[Tree], mut buffer: Vec<u32>,
                    maxx: f64, minx: f64, maxy: f64, miny: f64
) -> Vec<u32>{
    let mut changed_trees  = trees.iter().collect::<Vec<_>>();
    changed_trees.push(&new_tree);

    let mut ret_buffer = buffer.clone();

    let cluster_size = 9;

    for yy in (0..300).step_by(cluster_size) {
        for xx in (0..800).step_by(cluster_size) {
            let xxx = (xx as f64 / 800.) * (maxx - minx) + minx;
            let yyy = ((299 - yy) as f64 / 300.) * (maxy - miny) + miny;

            let mut value = 0;
            let dists_inv = changed_trees.iter().map(|tree| Point::new(
                tree.geom.get_point(0).0,
                tree.geom.get_point(0).1
            ).euclidean_distance(&Point::new(xxx, yyy)))
                .map(|x| 1./x as f32)
                .collect::<Vec<f32>>();

            let dists_total_inv: f32 = dists_inv.iter().sum::<f32>();
            // println!("{:?}", dists_total_inv);

            let mut value: f32 = 1. - dists_total_inv;
            for dy in 0..cluster_size {
                for dx in 0..cluster_size {
                    let x_idx = xx + dx;
                    let y_idx = yy + dy;
                    if x_idx < 800 && y_idx < 300 {
                        let i = (x_idx + y_idx * 800) as usize;
                        let mut color = buffer[i];
                        // let alpha = (color >> 24) & 0xFF;
                        let mut red = (color >> 16) & 0xFF;
                        let mut green = (color >> 8) & 0xFF;
                        let mut blue = color & 0xFF;

                        // println!("value: {}", value);


                        if value >= 0. && value <= 1. {
                            // Recolor increasingly red if the value is close to 1, or green if close to 0
                            // Scale red: as value increases, it becomes more red
                            // red = (red as f32 * (1. + (value * 255.))).min(255.0) as u32;
                            // Scale green: as value decreases, it becomes more green
                            // green = (green as f32 * (1. - (value * 255.))).min(255.0) as u32;
                            // Optionally adjust blue here if needed
                            // blue = (blue as f32 * (1. - (value * 255.))).min(255.0) as u32;
                        } else {
                            // If the value is out of range (less than 0 or greater than 1), max out green
                            // red = 255; // Maximize green
                            // println!("value under 0 or over 1");
                            // Optionally modify red or blue here depending on your requirements
                            // red = some_value;
                            // blue = some_value;
                        }

                        let diff_to_full_green= 255 - green;
                        let plus_green = (diff_to_full_green as f32 * value) as u32;
                        green = green + plus_green;

                        let diff_to_zero_red = red;
                        let minus_red = (diff_to_zero_red as f32 * value) as u32;
                        red = red - minus_red;

                        let diff_to_zero_blue = blue;
                        let minus_blue = (diff_to_zero_blue as f32 * value) as u32;
                        blue = blue - minus_blue;

                        // ret_buffer[i] = (red << 16) | (green << 8) | blue;
                        ret_buffer[i] = (red << 16) | (green << 8) | blue;
                    }
                }
            }

            // for y in changed_trees.iter() {
            //     if Point::new(xxx, yyy).euclidean_distance(
            //         &Point::new(y.geom.get_point(0).0, y.geom.get_point(0).1)
            //     ) < 30. {
            //         for dy in 0..cluster_size {
            //             for dx in 0..cluster_size {
            //                 let x_idx = xx + dx;
            //                 let y_idx = yy + dy;
            //                 if x_idx < 800 && y_idx < 300 {
            //                     let i = (x_idx + y_idx * 800) as usize;
            //                     let mut color = buffer[i];
            //                     let alpha = (color >> 24) & 0xFF;
            //                     let mut red = (color >> 16) & 0xFF;
            //                     let mut green = (color >> 8) & 0xFF;
            //                     let mut blue = color & 0xFF;
            //
            //                     red = (red as f32 * 2.).min(255.0) as u32;
            //                     green = (green as f32 * 0.5).min(255.0) as u32;
            //                     blue = (blue as f32 * 0.5).min(255.0) as u32;
            //
            //                     ret_buffer[i] = (alpha << 24) | (red << 16) | (green << 8) | blue;
            //                 }
            //             }
            //         }
            //     }
            // }
        }
    }

    ret_buffer
}

fn value_fn_step_30m(new_tree: Tree, trees: &[Tree]) -> usize {
    let mut proximity_count = 0;
    let new_tree_point = Point::new(new_tree.geom.get_point(0).0, new_tree.geom.get_point(0).1);
    for tree in trees {
        let tree_point = Point::new(tree.geom.get_point(0).0, tree.geom.get_point(0).1);
        // println!("dist: {:?}", new_tree_point.euclidean_distance(&tree_point));
        // println!("clicked: {:?}", new_tree_point);
        // println!("comparing to: {:?}", tree_point);
        if new_tree_point.euclidean_distance(&tree_point) < 30. {
            proximity_count += 1;
        }
    }
    proximity_count
}

#[derive(Clone)]
struct Tree {
    geom: Geometry, // Now we own the Geometry
}
