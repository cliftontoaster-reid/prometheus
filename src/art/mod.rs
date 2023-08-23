use image::{imageops::overlay, load_from_memory, DynamicImage, EncodableLayout, Pixel, Rgba};
use imageproc::drawing::draw_text_mut;
use rand::{seq::SliceRandom, thread_rng};
use rusttype::{point, Font, Scale};
use std::cmp::min;

pub fn background_image() -> DynamicImage {
  let raw: Vec<Vec<u8>> = vec![
    include_bytes!("images/background-0.png").to_vec(),
    include_bytes!("images/background-1.png").to_vec(),
    include_bytes!("images/background-2.png").to_vec(),
    include_bytes!("images/background-3.png").to_vec(),
  ];

  let mut rng: rand::rngs::ThreadRng = thread_rng();
  load_from_memory(raw.choose(&mut rng).unwrap()).unwrap()
}

pub async fn download_image(uri: &str, default: Vec<u8>) -> DynamicImage {
  load_from_memory(
    &reqwest::get(uri)
      .await
      .unwrap()
      .bytes()
      .await
      .unwrap_or(default.clone().into())
      .to_vec(),
  )
  .unwrap_or(load_from_memory(default.as_bytes()).unwrap())
}

pub async fn welcome(name: &str, server: &str, avatar: &str) -> DynamicImage {
  let mut base: DynamicImage = background_image();
  let mut avatar_img: DynamicImage =
    download_image(avatar, include_bytes!("images/avatar.png").to_vec())
      .await
      .resize_exact(512, 512, image::imageops::FilterType::Triangle);
  crop_to_circle(&mut avatar_img);

  overlay(&mut base, &avatar_img, 64, 64);
  let color = Rgba::from_slice(&[255u8, 255, 255, 255]).to_owned();
  let font = Font::try_from_bytes(include_bytes!("fonts/OldLondon.ttf")).unwrap();

  let t = cut_text_into_lines(&font, &format!("Greetings and salutations, noble Sir {}. With utmost reverence, I, as the devoted steward of our realm, extend a gracious welcome to thee in the name of our cherished domain, {}.", name, server), base.width() as f32 - 192.0, Scale { x: 36.0, y: 72.0 });

  for (index, text) in t.iter().rev().enumerate() {
    draw_text_mut(
      &mut base,
      color,
      64,
      1250 - 80 * index as i32,
      Scale { x: 36.0, y: 72.0 },
      &font,
      &text,
    );
  }

  base
}

#[tokio::test]
async fn test_welcome() {
  welcome("Amber Blackfire", "the source code", "https://cdn.discordapp.com/avatars/476782876755492868/7d14130cc654edf89c59d1f68a5780b5.png?size=1024").await.save("./test_welcome.png").unwrap()
}

pub fn crop_to_circle(image: &mut DynamicImage) -> &mut DynamicImage {
  let width = image.width();
  let height = image.height();
  let size = min(width, height);

  let mut result = image.clone().to_rgba8();

  for y in 0..size {
    for x in 0..size {
      let dx = x as i32 - size as i32 / 2;
      let dy = y as i32 - size as i32 / 2;

      if !(dx * dx + dy * dy <= (size as i32 / 2) * (size as i32 / 2)) {
        result.put_pixel(x, y, Rgba([0, 0, 0, 0]));
      }
    }
  }

  *image = DynamicImage::ImageRgba8(result);
  image
}

fn cut_text_into_lines(font: &Font, text: &str, max_width: f32, scale: Scale) -> Vec<String> {
  let mut lines = Vec::new();
  let mut current_line = String::new();
  let words = text.split_whitespace();

  for word in words {
    let new_line_width = measure_line(font, &(current_line.clone() + " " + word), scale).0;

    if new_line_width <= max_width {
      if !current_line.is_empty() {
        current_line.push(' ');
      }
      current_line.push_str(word);
    } else {
      lines.push(current_line.clone());
      current_line = word.to_string();
    }
  }

  if !current_line.is_empty() {
    lines.push(current_line);
  }

  lines
}

fn measure_line(font: &Font, text: &str, scale: Scale) -> (f32, f32) {
  let width = font
    .layout(text, scale, point(0.0, 0.0))
    .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
    .last()
    .unwrap_or(0.0);

  let v_metrics = font.v_metrics(scale);
  let height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

  (width, height)
}
