

#[macro_use]
extern crate vst;

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};


use std::sync::Arc;

use tuix::Application;

use tuix::{Entity, Event, State, BuildHandler, EventHandler, SliderEvent};


use tuix::widgets::value_knob::*;

static THEME: &str = include_str!("theme.css");

const WINDOW_WIDTH: usize = 300;
const WINDOW_HEIGHT: usize = 300;


struct GainWidget {
    control: Entity,
    params: Arc<GainEffectParameters>,
}

impl GainWidget {
    pub fn new(params: Arc<GainEffectParameters>) -> Self {
        GainWidget {
            control: Entity::null(),
            params: params.clone(),
        }
    }
}

impl BuildHandler for GainWidget {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        
        let val = self.params.amplitude.get();
        self.control = ValueKnob::new("Gain", val, 0.0, 1.0).build(state, entity, |builder| builder);
        
        entity
    }
}

impl EventHandler for GainWidget {
    fn on_event(&mut self, _state: &mut State, _entity: Entity, event: &mut Event) {

        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                SliderEvent::ValueChanged(val) => {
                    self.params.amplitude.set(*val);
                }

                _=> {}
            }
        }
    }
}

struct TestPluginEditor {
    params: Arc<GainEffectParameters>,
    is_open: bool,
}

impl Editor for TestPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        if self.is_open {
            return false;
        }

        self.is_open = true;

        let params = self.params.clone();

        Application::new(move |win_desc, state, window| {
            state.add_theme(THEME);

            GainWidget::new(params.clone()).build(state, window, |builder| {
                builder
            });
    
            win_desc.with_title("Hello Plugin").with_inner_size(300,300)
        }).open_parented(&VstParent(parent));

        true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}




struct GainEffectParameters {
    // The plugin's state consists of a single parameter: amplitude.
    amplitude: AtomicFloat,
}
struct TestPlugin {
    params: Arc<GainEffectParameters>,
    editor: Option<TestPluginEditor>,
}

impl Default for TestPlugin {
    fn default() -> Self {
        let params = Arc::new(GainEffectParameters::default());
        Self {
            params: params.clone(),
            editor: Some(TestPluginEditor {
                params: params.clone(),
                is_open: false,
            }),
        }
    }
}

impl Default for GainEffectParameters {
    fn default() -> GainEffectParameters {
        GainEffectParameters {
            amplitude: AtomicFloat::new(0.5),
        }
    }
}

impl Plugin for TestPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Tuix Gain Effect in Rust".to_string(),
            vendor: "Geom3trik".to_string(),
            unique_id: 243123073,
            version: 1,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: 1,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        // let log_folder = ::dirs::home_dir().unwrap().join("tmp");

        // let _ = ::std::fs::create_dir(log_folder.clone());

        // let log_file = ::std::fs::File::create(log_folder.join("TuixBaseviewTest.log")).unwrap();

        // let log_config = ::simplelog::ConfigBuilder::new()
        //     .set_time_to_local(true)
        //     .build();

        // let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file);

        // ::log_panics::init();

        // ::log::info!("init");
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // Read the amplitude from the parameter object
        let amplitude = self.params.amplitude.get();
        // First, we destructure our audio buffer into an arbitrary number of
        // input and output buffers.  Usually, we'll be dealing with stereo (2 of each)
        // but that might change.
        for (input_buffer, output_buffer) in buffer.zip() {
            // Next, we'll loop through each individual sample so we can apply the amplitude
            // value to it.
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = *input_sample * amplitude;
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for GainEffectParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.amplitude.get(),
            _ => 0.0,
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        match index {
            0 => self.amplitude.set(val),
            _ => (),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.2}", (self.amplitude.get() - 0.5) * 2f32),
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Amplitude",
            _ => "",
        }
        .to_string()
    }
}

struct VstParent(*mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}

plugin_main!(TestPlugin);