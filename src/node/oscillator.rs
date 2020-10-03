use dasp_signal::{self as signal, Signal};
use dasp_graph::{Buffer, Input, Node, NodeData, BoxedNodeSend};
use pest::iterators::Pairs;
use super::super::Rule;

pub struct SinOsc {
    // pub freq: f64,
    // pub sig: Sine<ConstHz>
    freq: f32,
    phase: f32,
    diff: f32,
    has_mod: bool
    // pub sig: Box<dyn Signal<Frame=f64> + Send>,
}

impl SinOsc {
    pub fn new(paras: &mut Pairs<Rule>) -> (NodeData<BoxedNodeSend>, Vec<String>) {
        let mut paras = paras.next().unwrap().into_inner();
        let freq: String = paras.next().unwrap().as_str().to_string()
        .chars().filter(|c| !c.is_whitespace()).collect();

        if freq.parse::<f32>().is_ok() {
            return (NodeData::new1(BoxedNodeSend::new(Self {
                freq: freq.parse::<f32>().unwrap(),
                phase: 0.0, diff: 0.0, has_mod: false 
            })), vec![])
        } else {
            return (NodeData::new1(BoxedNodeSend::new(Self { 
                freq: 0.0,
                phase:  0.0, diff: 0.0, has_mod: true 
            })), vec![freq])
        }
    }
}

impl Node for SinOsc {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        
        // let freq = self.freq.parse::<f32>();
        if self.has_mod {
            // assert_eq!(inputs.len(), 1);
            assert!(inputs.len() > 0);
            let mod_buf = &mut inputs[0].buffers();
            for i in 0..64 {
                output[0][i] = (self.phase * 2.0 * std::f32::consts::PI).sin();
                if mod_buf[0][i] != 0.0 { // doesn't make sense to have 0 freq
                    self.diff = mod_buf[0][i] / 44100.0;    
                }
                self.phase += self.diff;
                // self.phase += 440.0 / 44100.0;
                if self.phase > 1.0 {
                    self.phase -= 1.0
                }
            }
            // }
        } else {

            for i in 0..64 {
                output[0][i] = self.phase.sin();
                self.phase += self.freq / 44100.0 * 2.0 * std::f32::consts::PI;
                // self.phase += 220.0 / 44100.0;
                if self.phase > 2.0 * std::f32::consts::PI {
                    self.phase -= 2.0 * std::f32::consts::PI
                }
            }
        }
    }
}

pub struct Impulse {
    sig: Box<dyn Signal<Frame=f32> + Send>,
    // sig: GenMut<(dyn Signal<Frame=f32> + 'static + Sized), f32>
}

impl Impulse {
    pub fn new(paras: &mut Pairs<Rule>) -> (NodeData<BoxedNodeSend>, Vec<String>) {
        let para_a: String = paras.next().unwrap().as_str().to_string()
        .chars().filter(|c| !c.is_whitespace()).collect();

        assert!(para_a.parse::<f32>().is_ok(), "parameter not a float");

        let freq = para_a.parse::<f32>().unwrap();
        let p = (44100.0 / freq) as usize;
        let mut i: usize = 0;
        let s = signal::gen_mut(move || {
            let imp = (i % p == 0) as u8;
            i += 1;
            imp as f32
        });
        (NodeData::new1(BoxedNodeSend::new(Self {
            sig: Box::new(s)
        })), vec![])
    }
}

impl Node for Impulse {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        for o in output {
            o.iter_mut().for_each(|s| *s = self.sig.next() as f32);
        }
    }
}

pub struct Saw {
    phase_n: usize,
    freq: f32,
    has_sidechain: bool
}

impl Saw {
    pub fn new(paras: &mut Pairs<Rule>) -> (NodeData<BoxedNodeSend>, Vec<String>) {
        let para_a: String = paras.next().unwrap().as_str().to_string()
        .chars().filter(|c| !c.is_whitespace()).collect();

        let is_float = para_a.parse::<f32>();
        let has_sidechain = !is_float.is_ok();
        let (freq, sidechain) = match has_sidechain {
            true => (0.0,vec![para_a]),
            false => (is_float.unwrap(), vec![])
        };

        (NodeData::new1(BoxedNodeSend::new(Self {
            phase_n: 0,
            freq: freq,
            has_sidechain: has_sidechain
        })), sidechain)
    }
}

impl Node for Saw {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {

        for i in 0..64 {
            if self.has_sidechain {
                let mod_buf = &mut inputs[0].buffers();
                if mod_buf[0][i] != 0.0 {
                    self.freq = mod_buf[0][i];
                }   
            }
            let circle_len = (44100.0 / self.freq) as usize;
            output[0][i] = ((self.phase_n % circle_len) as f32 / circle_len as f32)*2.0-0.5;
            self.phase_n += 1;
        }
    }
}

pub struct Square {
    phase_n: usize,
    freq: f32,
    has_sidechain: bool
}

impl Square {
    pub fn new(paras: &mut Pairs<Rule>) -> (NodeData<BoxedNodeSend>, Vec<String>) {
        let para_a: String = paras.next().unwrap().as_str().to_string()
        .chars().filter(|c| !c.is_whitespace()).collect();

        let is_float = para_a.parse::<f32>();
        let has_sidechain = !is_float.is_ok();
        let (freq, sidechain) = match has_sidechain {
            true => (0.0,vec![para_a]),
            false => (is_float.unwrap(), vec![])
        };

        (NodeData::new1(BoxedNodeSend::new(Self {
            phase_n: 0,
            freq: freq,
            has_sidechain: has_sidechain
        })), sidechain)
    }
}

impl Node for Square {
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        for i in 0..64 {
            if self.has_sidechain {
                let mod_buf = &mut inputs[0].buffers();
                if mod_buf[0][i] != 0.0 {
                    self.freq = mod_buf[0][i];
                }   
            }
            let circle_len = (44100.0 / self.freq) as usize;
            output[0][i] = ((self.phase_n % circle_len) > (circle_len / 2)) as u8 as f32;
            self.phase_n += 1;
        }
    }
}