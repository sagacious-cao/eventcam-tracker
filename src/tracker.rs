


use arcstar::sae_types::*;

use graphics_buffer::*;

use rand;

use crate::slab::{SlabStore};

pub struct FeatureTracker {
    store: SlabStore,
}

impl FeatureTracker {
  pub const RAINBOW_WHEEL_DIM: usize = 12;
  pub const RAINBOW_WHEEL_GEN: [[u8; 3]; Self::RAINBOW_WHEEL_DIM] = [
    [255, 0, 0],
    [255, 127, 0],
    [255, 255, 0],
    [127, 255, 0],
    [0, 255, 0],
    [0, 255, 127],
    [0, 255, 255],
    [0, 127, 255],
    [0, 0, 255],
    [127, 0, 255],
    [255, 0, 255],
    [255, 0, 127],
  ];

  pub fn new() -> FeatureTracker {
    FeatureTracker {
      store: SlabStore::new(),
    }
  }

  pub fn add_and_match_feature(&mut self, new_evt: &SaeEvent) -> Option<SaeEvent> {
    let res = self.store.add_and_match_feature(new_evt);
    match res {
      Some(feat) => Some(feat.event),
      None => None
    }
  }


  fn render_track(segments: &Vec<[f64; 4]>, buffer: &mut RenderBuffer, color_hint: usize) {
    for seg in segments {
      Self::render_one_line(*seg, buffer, color_hint);
    }
  }

  fn render_one_line(line: [f64; 4], buffer: &mut RenderBuffer, color_hint: usize) {
    let color_idx = (color_hint * (FeatureTracker::RAINBOW_WHEEL_DIM / 3) + color_hint) % FeatureTracker::RAINBOW_WHEEL_DIM;
    let rgb_data = FeatureTracker::RAINBOW_WHEEL_GEN[color_idx];
    let px: [f32; 4] = [
      (rgb_data[0] as f32) / 255.0,
      (rgb_data[1] as f32) / 255.0,
      (rgb_data[2] as f32) / 255.0,
      1.0 //alpha
    ];
    graphics::line(
      px,
      1.0,
      line,
      IDENTITY,
      buffer
    );
  }


  /// Render lines for all the valid tracks to the given file path
  pub fn render_tracks_to_file(&self, nrows: u32, ncols: u32, lead_time_horizon: SaeTime, out_path: &str) {
    let mut buffer = RenderBuffer::new(ncols, nrows);
    buffer.clear([0.0, 0.0, 0.0, 1.0]);

    let chains_list: Vec<Vec<[f64; 4]>> = (0..nrows * ncols).fold(vec!(), |mut acc, idx| {
      let row: u16 = (idx / ncols) as u16;
      let col: u16 = (idx % ncols) as u16;
      let chain = self.store.chain_for_point(row, col, 0);
      if chain.len() > 2 { //TODO arbitrary cutoff
        let lead_evt = &chain[0];
        let mut total_dist_x = 0;
        let mut total_dist_y = 0;

        if lead_evt.timestamp >= lead_time_horizon {
          //add all events in the chain
          let mut chain_vec: Vec<[f64; 4]> = vec!();
          for i in 0..(chain.len() - 1) {
            let evt = &chain[i];
            let old_evt = &chain[i + 1];
            let dist_x = (evt.col as i32) - (old_evt.col as i32);
            let dist_y = (evt.row as i32) - (old_evt.row as i32);
            total_dist_x += dist_x;
            total_dist_y += dist_y;


            let le_line: [f64; 4] = [evt.col as f64, evt.row as f64, old_evt.col as f64, old_evt.row as f64];
            chain_vec.push(le_line);
          }
          acc.push(chain_vec);

          //if 0 == total_dist_y {
          //  println!("total_dist xy: {}, {}", total_dist_x, total_dist_y);
          //}
        }
      }
      acc
    });

    println!("rendering {} lines", chains_list.len());
    for (i, chain) in chains_list.iter().enumerate() {
      Self::render_track(chain, &mut buffer, i);
    }


    buffer.save(out_path).expect("Couldn't save");
  }


//  //squiggle that triggers corner detector, but not clear what this corresponds to IRL
//  const SAMPLE_CORNER_DIM: usize = 8;
//  const SAMPLE_CORNER_GEN: [[i32; 2]; Self::SAMPLE_CORNER_DIM] = [
//    [1, -4], [2, -3], [3, -2], [4, -1],
//    [1, -3], [2, -2], [3, -1],
//    [0, 0]
//  ];

//  //true corner L
//  const SAMPLE_CORNER_DIM: usize = 10;
//  const SAMPLE_CORNER_GEN: [[i32; 2]; SAMPLE_CORNER_DIM] = [
//    [4, 0], [3,0], [2,0], [1,0],
//    [1,-1],
//    [0, -4], [0, -3], [0, -2], [0, -1],
//    [0, 0]
//  ];


  /*
  /// Insert a synthetic event at the given position
  /// append the synthesized event to the modified event list
  pub fn insert_one_synth_event(ctr_row: i32, ctr_col: i32, timestamp: &mut SaeTime, event_list: &mut Vec<SaeEvent> ) {
    for j in 0..Self::SAMPLE_CORNER_DIM {
      *timestamp += TIME_INCR;
      let dxy = Self::SAMPLE_CORNER_GEN[j];
      let evt_row = ctr_row + dxy[1];
      let evt_col = ctr_col + dxy[0];

      let mut rng = rand::thread_rng();
      let mut ndesc:NormDescriptor = [0.0; NORM_DESCRIPTOR_LEN];

      for i in 0..ndesc.len() {
        ndesc[i] = rng.gen::<f32>();
      }

      let evt = SaeEvent {
        row: evt_row as u16,
        col: evt_col as u16,
        polarity: 1,
        timestamp: *timestamp,
        norm_descriptor: Some(Box::new(ndesc)),
      };

      event_list.push(evt);
    }
  }
  */

/*
  /// Generate a series of synthetic events for stimulating the tracker
  pub fn process_synthetic_events(img_w: u32, img_h: u32, render_out: bool) {
    let mut tracker = Box::new(FeatureTracker::new());
    let mut timestamp: SaeTime = TIME_INCR;

    // The Surface of Active Events (timestamps for last event at each pixel point)
    let mut sae_rise = SaeMatrix::zeros(
      img_h as usize, // rows
      img_w as usize // cols
    );

    let mut sae_fall = SaeMatrix::zeros(
      img_h as usize, // rows
      img_w as usize // cols
    );

    let mut ctr_col: i32 = (img_w as i32) - (2 * DCONN as i32);
    let ctr_row: i32 = (img_w / 2) as i32;
    let mut chunk_count = 0;

    //these three constants in essence define the minimum distinguishable corner grid
    const COL_DECR: i32 = 2;
    let row_gap: i32 = 2 * DCONN as i32;
    let col_gap: i32 = 2 * DCONN as i32;


    let half_width: i32 = (img_w / 2) as i32;
    let total_frames = half_width / COL_DECR;

    for _i in 0..total_frames {
      let mut event_list: Vec<SaeEvent> = vec!();
      chunk_count += 1;

      Self::insert_one_synth_event(ctr_row - row_gap, ctr_col, &mut timestamp, &mut event_list);
      Self::insert_one_synth_event(ctr_row, ctr_col, &mut timestamp, &mut event_list);
      Self::insert_one_synth_event(ctr_row + row_gap, ctr_col, &mut timestamp, &mut event_list);

      Self::insert_one_synth_event(ctr_row - row_gap, ctr_col - col_gap, &mut timestamp, &mut event_list);
      Self::insert_one_synth_event(ctr_row, ctr_col - col_gap, &mut timestamp, &mut event_list);
      Self::insert_one_synth_event(ctr_row + row_gap, ctr_col - col_gap, &mut timestamp, &mut event_list);

      let matches = process_events(&mut tracker, &mut sae_rise, &mut sae_fall, &event_list);

      if render_out {
        let lead_events = matches.iter().map(|(new, _old)| new.clone()).collect();
        let out_img = render_corners(img_h, img_w, &lead_events);
        let out_path = format!("./out/sae_{:04}_evts.png", chunk_count);
        //flame::start("save_image");
        out_img.save(out_path).expect("Couldn't save");
        //flame::end("save_image");
      }

      let out_path = format!("./out/sae_{:04}_tracks.png", chunk_count);

      let mut minimum_lead_event_time = SaeTime::max_value();
      for (new_evt, _old_evt) in matches {
        if new_evt.timestamp < minimum_lead_event_time {
          minimum_lead_event_time = new_evt.timestamp;
        }
      }
      if render_out {
        tracker.render_tracks_to_file(img_h, img_w, minimum_lead_event_time, &out_path);
      }

      ctr_col -= COL_DECR;
    }
  }
*/

}


//
//#[cfg(test)]
//mod tests {
//    use super::*;
//x
//    #[test]
//    fn test_find_neighbors() {
//        let mut tracker = FeatureTracker::new();
//        //test empty tracker
//    }
//}