use std::num::NonZeroU32;
use std::rc::Rc;

use anyhow::Context;
use rweb::browser::Browser;
use rweb::browser::BrowserEvent;
use rweb::browser::Loader;
use rweb::browser::Renderer;
use rweb::browser::VSTEP;
use rweb::browser::WINDOW_HEIGHT;
use rweb::browser::WINDOW_WIDTH;
use rweb::loader::Url;
use softbuffer::Context as SoftbufferContext;
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::OwnedDisplayHandle;
use winit::keyboard::Key;
use winit::keyboard::NamedKey;
use winit::window::Window;
use winit::window::WindowId;

pub fn show(title: &str, url: Url) -> anyhow::Result<()> {
    let event_loop: EventLoop<BrowserEvent> = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let context = SoftbufferContext::new(event_loop.owned_display_handle()).map_err(gui_error)?;
    let browser = Browser::new(url.clone());
    let loader = Loader::new(event_loop.create_proxy())?;
    let mut app = App::new(context, title.to_owned(), browser, loader, url);

    event_loop.run_app(&mut app)?;

    Ok(())
}

struct App {
    context: SoftbufferContext<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    title: String,
    browser: Browser,
    loader: Loader,
    initial_url: Option<Url>,
}

impl App {
    fn new(
        context: SoftbufferContext<OwnedDisplayHandle>,
        title: String,
        browser: Browser,
        loader: Loader,
        initial_url: Url,
    ) -> Self {
        Self {
            context,
            window: None,
            surface: None,
            title,
            browser,
            loader,
            initial_url: Some(initial_url),
        }
    }

    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        let window = self.window.as_ref().context("missing window")?;
        let size = window.inner_size();

        let Some(width) = NonZeroU32::new(size.width) else {
            return Ok(());
        };
        let Some(height) = NonZeroU32::new(size.height) else {
            return Ok(());
        };

        let surface = self.surface.as_mut().context("missing surface")?;
        surface.resize(width, height).map_err(gui_error)?;

        let mut buffer = surface.buffer_mut().map_err(gui_error)?;
        let width = buffer.width().get();
        let height = buffer.height().get();

        self.browser.layout_active_page(width as i32, height as i32);
        Renderer::draw(
            &mut buffer,
            width,
            height,
            self.browser.active_display_list(),
            self.browser.active_scroll_y(),
        );
        buffer.present().map_err(gui_error)?;

        Ok(())
    }

    fn scroll(&mut self, delta: MouseScrollDelta) {
        let amount = match delta {
            MouseScrollDelta::LineDelta(_, y) => -(y * VSTEP as f32 * 3.0) as i32,
            MouseScrollDelta::PixelDelta(position) => -position.y as i32,
        };

        self.browser.scroll_active_page(amount);
    }

    fn start_initial_load(&mut self) {
        if let Some(url) = self.initial_url.take() {
            self.browser.start_navigation(url.clone());
            self.loader.load(url);
        }
    }
}

fn gui_error(err: impl std::fmt::Debug) -> anyhow::Error {
    anyhow::anyhow!("{err:?}")
}

impl ApplicationHandler<BrowserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            panic!("window already exists");
        }

        let attrs = Window::default_attributes()
            .with_title(format!("rweb - {}", self.title))
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        let window = Rc::new(event_loop.create_window(attrs).unwrap());
        let surface = Surface::new(&self.context, Rc::clone(&window)).unwrap();

        self.surface = Some(surface);
        self.window = Some(window);
        self.start_initial_load();
        self.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: BrowserEvent) {
        match event {
            BrowserEvent::PageLoaded(result) => {
                self.browser.finish_navigation(result);
                self.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let is_current_window = self
            .window
            .as_ref()
            .is_some_and(|window| window.id() == window_id);
        if !is_current_window {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.draw() {
                    eprintln!("failed to draw window: {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => self.request_redraw(),
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll(delta);
                self.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
                self.request_redraw();
            }
            _ => {}
        }
    }
}

impl App {
    fn handle_keyboard_input(&mut self, event: KeyEvent) {
        match event.state {
            ElementState::Pressed => match event.logical_key {
                Key::Named(NamedKey::ArrowDown) => {
                    self.browser.scroll_active_page(VSTEP * 3);
                    self.request_redraw();
                }
                Key::Named(NamedKey::ArrowUp) => {
                    self.browser.scroll_active_page(-VSTEP * 3);
                    self.request_redraw();
                }

                _ => {}
            },

            ElementState::Released => {}
        }
    }
}
