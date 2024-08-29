use iced::widget::canvas::Frame;
use iced::widget::canvas::Path;
use iced::widget::canvas::Program;
use iced::widget::canvas::Stroke;

use iced::Color;
use iced::Point;
use iced::Size;

use porter_math::Angles;
use porter_math::Quaternion;
use porter_math::Vector3;

/// The size of the cube in pixels.
const CUBE_SIZE: f32 = 50.;

/// A canvas renderer for the splash screen right side.
pub struct PorterSplash(pub f32);

impl<Message> Program<Message> for PorterSplash {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let mut offset_x = 0.0;
        let mut offset_y = 0.0;

        let Size { width, height } = frame.size();

        let stroke = Stroke::default()
            .with_color(Color::from_rgba(0.2, 0.2, 0.2, 0.2))
            .with_width(1.0);

        let stroke_full = Stroke::default()
            .with_color(Color::from_rgba(0.2, 0.2, 0.2, 0.5))
            .with_width(1.0);

        while offset_x <= width || offset_y <= height {
            let stroke = if (((offset_x / 25.0) as u32) % 4) == 3 {
                stroke_full.clone()
            } else {
                stroke.clone()
            };

            frame.stroke(
                &Path::line(Point::new(offset_x, 0.0), Point::new(offset_x, height)),
                stroke.clone(),
            );

            frame.stroke(
                &Path::line(Point::new(0.0, offset_y), Point::new(width, offset_y)),
                stroke,
            );

            offset_x += 25.0;
            offset_y += 25.0;
        }

        let center = bounds.center();

        let rotation_x =
            Quaternion::from_axis_rotation(Vector3::new(1.0, 0.0, 0.0), self.0, Angles::Degrees)
                .to_4x4();

        let rotation_z =
            Quaternion::from_axis_rotation(Vector3::new(0.0, 0.0, 1.0), self.0, Angles::Degrees)
                .to_4x4();

        let mut vertices = [
            Vector3::new(-CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(CUBE_SIZE, -CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(-CUBE_SIZE, CUBE_SIZE, -CUBE_SIZE),
            Vector3::new(-CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE),
            Vector3::new(CUBE_SIZE, -CUBE_SIZE, CUBE_SIZE),
            Vector3::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
            Vector3::new(-CUBE_SIZE, CUBE_SIZE, CUBE_SIZE),
        ];

        for vertex in &mut vertices {
            *vertex = vertex.transform(&rotation_z);
            *vertex = vertex.transform(&rotation_x);
            *vertex += Vector3::new(center.x / 2.0, center.y, 0.0);
        }

        let mut faces = [
            ([vertices[0], vertices[1], vertices[5], vertices[4]], 0),
            ([vertices[1], vertices[2], vertices[6], vertices[5]], 1),
            ([vertices[2], vertices[3], vertices[7], vertices[6]], 2),
            ([vertices[3], vertices[0], vertices[4], vertices[7]], 3),
            ([vertices[0], vertices[1], vertices[2], vertices[3]], 4),
            ([vertices[4], vertices[5], vertices[6], vertices[7]], 5),
        ];

        faces.sort_by(|x, y| {
            let x_depth = x.0.iter().map(|x| x.z).sum::<f32>() / 4.0;
            let y_depth = y.0.iter().map(|x| x.z).sum::<f32>() / 4.0;

            x_depth.total_cmp(&y_depth)
        });

        for (vertices, i) in faces {
            let path = Path::new(|b| {
                for vertex in vertices {
                    b.line_to(Point::new(vertex.x, vertex.y));
                }
                b.close();
            });

            let color = match i {
                0 => Color::from_rgb8(0x07, 0x7B, 0xB4),
                1 => Color::from_rgb8(0x37, 0xAB, 0xE4),
                2 => Color::from_rgb8(0x07, 0x7B, 0xB4),
                3 => Color::from_rgb8(0x37, 0xAB, 0xE4),
                _ => Color::from_rgb8(0x27, 0x9B, 0xD4),
            };

            frame.fill(&path, color);
        }

        vec![frame.into_geometry()]
    }
}
