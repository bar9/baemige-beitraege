use gdal::Dataset;
use gdal::vector::{Geometry, LayerAccess};
use geo::{EuclideanDistance, Point};
use minifb::{Window, WindowOptions, Key, MouseButton, MouseMode};

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
                    let trees: Vec<Tree> = layer.features().filter_map(|feature| {
                        if let Some(geometry) = feature.geometry() {
                            Some(Tree { geom: geometry.clone() })
                        } else {
                            None
                        }
                    }).collect();

                    // Initialize the window for rendering
                    let mut window = Window::new("Shapefile Viewer", 800, 600, WindowOptions {
                        resize: true,
                        ..Default::default()
                    }).unwrap();

                    let mut mouse_down_last_frame = false;

                    // Main loop for window and event handling
                    while window.is_open() && !window.is_key_down(Key::Escape) {
                        // Handle mouse click
                        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) {
                            let mouse_down = window.get_mouse_down(MouseButton::Left);
                            if mouse_down && !mouse_down_last_frame {
                                // Transform pixel coordinates (x, y) to spatial coordinates
                                let spatial_x = minx + (x as f64 / 800.0) * (maxx - minx);
                                let spatial_y = maxy - (y as f64 / 600.0) * (maxy - miny);

                                let click_pos = Point::new(spatial_x, spatial_y);
                                let new_tree = Tree {
                                    geom: Geometry::from_wkt(&format!("POINT({} {})", spatial_x, spatial_y)).unwrap(),
                                };

                                // Call the algorithm_outline function
                                let result = value_fn_step_30m(new_tree, &trees);
                                println!("algorithm_outline result: {}", result);
                            }

                            // Update the mouse state for the next frame
                            mouse_down_last_frame = mouse_down;
                        }

                        // Render the map (simplified)
                        let mut buffer: Vec<u32> = vec![0; 800 * 600]; // Window size (800x600)
                        for tree in &trees {
                            // Simplified rendering of tree locations as red points
                            let (tx, ty) = (tree.geom.get_point(0).0 as i32, tree.geom.get_point(0).1 as i32);
                            if tx >= 0 && tx < 800 && ty >= 0 && ty < 600 {
                                buffer[(ty as usize * 800 + tx as usize)] = 0xFF0000; // Red color
                            }
                        }

                        window.update_with_buffer(&buffer, 800, 600).unwrap();
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
