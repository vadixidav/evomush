use QBezier;
use std::iter::once;

const CIRCLE_BAND_RADIUS: f32 = 0.3;

pub fn make_circle(color: [f32; 4]) -> impl Iterator<Item=QBezier> {
    once(QBezier {
         position0: [0.0, -1.0],
         position1: [0.5773502691896256, -1.0],
         position2: [0.8660254037844386, -0.5],
         inner_color0: color,
         inner_color1: color,
         falloff_color0: color,
         falloff_color1: color,
         falloff0: 0.25,
         falloff1: 0.25,
         falloff_radius0: CIRCLE_BAND_RADIUS,
         falloff_radius1: CIRCLE_BAND_RADIUS,
         inner_radius0: 0.0,
         inner_radius1: 0.0,
     }).chain(once(
     QBezier {
         position0: [0.8660254037844386, -0.5],
         position1: [1.1547005383792515, 0.0],
         position2: [0.8660254037844387, 0.5],
         inner_color0: color,
         inner_color1: color,
         falloff_color0: color,
         falloff_color1: color,
         falloff0: 0.25,
         falloff1: 0.25,
         falloff_radius0: CIRCLE_BAND_RADIUS,
         falloff_radius1: CIRCLE_BAND_RADIUS,
         inner_radius0: 0.0,
         inner_radius1: 0.0,
     }).chain(once(
     QBezier {
         position0: [0.8660254037844387, 0.5],
         position1: [0.5773502691896261, 1.0],
         position2: [0.0, 1.0],
         inner_color0: color,
         inner_color1: color,
         falloff_color0: color,
         falloff_color1: color,
         falloff0: 0.25,
         falloff1: 0.25,
         falloff_radius0: CIRCLE_BAND_RADIUS,
         falloff_radius1: CIRCLE_BAND_RADIUS,
         inner_radius0: 0.0,
         inner_radius1: 0.0,
     }).chain(once(
     QBezier {
         position0: [0.0, 1.0],
         position1: [-0.5773502691896254, 1.0],
         position2: [-0.8660254037844384, 0.5],
         inner_color0: color,
         inner_color1: color,
         falloff_color0: color,
         falloff_color1: color,
         falloff0: 0.25,
         falloff1: 0.25,
         falloff_radius0: CIRCLE_BAND_RADIUS,
         falloff_radius1: CIRCLE_BAND_RADIUS,
         inner_radius0: 0.0,
         inner_radius1: 0.0,
     }).chain(once(
     QBezier {
         position0: [-0.8660254037844384, 0.5],
         position1: [-1.1547005383792515, 0.0],
         position2: [-0.866025403784439, -0.5],
         inner_color0: color,
         inner_color1: color,
         falloff_color0: color,
         falloff_color1: color,
         falloff0: 0.25,
         falloff1: 0.25,
         falloff_radius0: CIRCLE_BAND_RADIUS,
         falloff_radius1: CIRCLE_BAND_RADIUS,
         inner_radius0: 0.0,
         inner_radius1: 0.0,
     }).chain(once(
     QBezier {
         position0: [-0.866025403784439, -0.5],
         position1: [-0.5773502691896263, -1.0],
         position2: [-0.0, -1.0],
         inner_color0: color,
         inner_color1: color,
         falloff_color0: color,
         falloff_color1: color,
         falloff0: 0.25,
         falloff1: 0.25,
         falloff_radius0: CIRCLE_BAND_RADIUS,
         falloff_radius1: CIRCLE_BAND_RADIUS,
         inner_radius0: 0.0,
         inner_radius1: 0.0,
     }))))))
}
