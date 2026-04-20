use crate::{
    components::icons::{StaticIcon, icon_button},
    components::{ButtonSize, MenuSize},
    components::position_button,
    config::MediaPlayerModuleConfig,
    services::{
        ReadOnlyService, Service, ServiceEvent,
        mpris::{
            MprisPlayerCommand, MprisPlayerData, MprisPlayerService, PlaybackStatus, PlayerCommand,
        },
    },
    theme::AshellTheme,
    utils::truncate_text,
};
use crate::components::ButtonUIRef;
use iced::{
    Background, Border, Element, Length, Subscription, SurfaceId, Task,
    Theme,
    alignment::Vertical,
    widget::{column, container, image, row, slider, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    Prev(String),
    PlayPause(String),
    Next(String),
    SetVolume(String, f64),
    Event(ServiceEvent<MprisPlayerService>),
    ConfigReloaded(MediaPlayerModuleConfig),
    OpenMenu(SurfaceId, ButtonUIRef),
}

pub enum Action {
    None,
    Command(Task<Message>),
    OpenMenu(SurfaceId, ButtonUIRef),
}

pub struct MediaPlayerP {
    config: MediaPlayerModuleConfig,
    service: Option<MprisPlayerService>,
}

impl MediaPlayerP {
    pub fn new(config: MediaPlayerModuleConfig) -> Self {
        Self {
            config,
            service: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Prev(s) => Action::Command(self.handle_command(s, PlayerCommand::Prev)),
            Message::PlayPause(s) => {
                Action::Command(self.handle_command(s, PlayerCommand::PlayPause))
            }
            Message::Next(s) => Action::Command(self.handle_command(s, PlayerCommand::Next)),
            Message::SetVolume(s, v) => {
                Action::Command(self.handle_command(s, PlayerCommand::Volume(v)))
            }
            Message::Event(event) => match event {
                ServiceEvent::Init(s) => {
                    self.service = Some(s);
                    Action::None
                }
                ServiceEvent::Update(d) => {
                    if let Some(service) = self.service.as_mut() {
                        service.update(d);
                    }
                    Action::None
                }
                ServiceEvent::Error(_) => Action::None,
            },
            Message::ConfigReloaded(c) => {
                self.config = c;
                Action::None
            }
            Message::OpenMenu(id, button_ui_ref) => Action::OpenMenu(id, button_ui_ref),
        }
    }

    pub fn view<'a>(&'a self, id: SurfaceId, theme: &'a AshellTheme) -> Option<Element<'a, Message>> {
        self.service.as_ref().and_then(|s| {
            s.players().first().map(|player| {
                let play_pause_icon = match player.state {
                    PlaybackStatus::Playing => StaticIcon::Pause,
                    PlaybackStatus::Paused | PlaybackStatus::Stopped => StaticIcon::Play,
                };

                let service_name = player.service.clone();
                let service_name_next = player.service.clone();

                let title_text = truncate_text(
                    &self.get_title(player),
                    self.config.max_title_length,
                );

                let title_btn = position_button(
                    text(title_text)
                        .size(theme.font_size.sm)
                        .wrapping(text::Wrapping::None),
                )
                .padding([2.0, theme.space.xs])
                .style(theme.module_button_style())
                .on_press_with_position(move |button_ui_ref| {
                    Message::OpenMenu(id, button_ui_ref)
                });

                row![
                    icon_button(theme, StaticIcon::SkipPrevious)
                        .on_press(Message::Prev(service_name.clone()))
                        .size(ButtonSize::Small),
                    icon_button(theme, play_pause_icon)
                        .on_press(Message::PlayPause(service_name))
                        .size(ButtonSize::Small),
                    title_btn,
                    icon_button(theme, StaticIcon::SkipNext)
                        .on_press(Message::Next(service_name_next))
                        .size(ButtonSize::Small),
                ]
                .align_y(Vertical::Center)
                .spacing(theme.space.xxs)
                .into()
            })
        })
    }

    pub fn menu_view<'a>(&'a self, _id: SurfaceId, theme: &'a AshellTheme) -> Element<'a, Message> {
        container(match &self.service {
            None => Into::<Element<'a, Message>>::into(text("Not connected to MPRIS service")),
            Some(service) => column!(
                column(service.players().iter().map(|d| {
                    const LEFT_COLUMN_WIDTH: Length = Length::FillPortion(3);
                    const RIGHT_COLUMN_WIDTH: Length = Length::FillPortion(2);
                    let m = d.metadata.as_ref();
                    let title_str = m
                        .and_then(|m| m.title.clone())
                        .unwrap_or("No Title".to_string());
                    let artists = m
                        .and_then(|m| m.artists.clone())
                        .map(|a| a.join(", "))
                        .unwrap_or("Unknown Artist".to_string());
                    let album = m
                        .and_then(|m| m.album.clone())
                        .unwrap_or("Unknown Album".to_string());
                    let title_widget = text(truncate_text(&title_str, self.config.max_title_length))
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .width(Length::Fill);
                    let artists_widget = text(truncate_text(&artists, self.config.max_title_length))
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .size(theme.font_size.sm)
                        .width(Length::Fill);
                    let album_widget = text(truncate_text(&album, self.config.max_title_length))
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .size(theme.font_size.sm)
                        .width(Length::Fill);
                    let description = column![title_widget, artists_widget, album_widget]
                        .spacing(theme.space.xxs)
                        .width(LEFT_COLUMN_WIDTH);

                    let play_pause_icon = match d.state {
                        PlaybackStatus::Playing => StaticIcon::Pause,
                        PlaybackStatus::Paused | PlaybackStatus::Stopped => StaticIcon::Play,
                    };

                    let buttons = container(
                        row![
                            icon_button(theme, StaticIcon::SkipPrevious)
                                .on_press(Message::Prev(d.service.clone()))
                                .size(ButtonSize::Large),
                            icon_button(theme, play_pause_icon)
                                .on_press(Message::PlayPause(d.service.clone()))
                                .size(ButtonSize::Large),
                            icon_button(theme, StaticIcon::SkipNext)
                                .on_press(Message::Next(d.service.clone()))
                                .size(ButtonSize::Large),
                        ]
                        .align_y(Vertical::Center)
                        .spacing(theme.space.xs),
                    )
                    .center_x(RIGHT_COLUMN_WIDTH);

                    let volume_slider: Option<Element<'_, _>> = d.volume.map(|v| {
                        slider(0.0..=100.0, v, move |v| {
                            Message::SetVolume(d.service.clone(), v)
                        })
                        .width(LEFT_COLUMN_WIDTH)
                        .into()
                    });

                    let cover: Option<Element<'_, _>> = d
                        .metadata
                        .as_ref()
                        .and_then(|meta| meta.art_url.as_ref())
                        .map(|url| {
                            let inner: Element<'_, _> = service
                                .get_cover(url)
                                .map(|handle| {
                                    image(handle)
                                        .filter_method(image::FilterMethod::Linear)
                                        .into()
                                })
                                .unwrap_or_else(|| text("Loading cover...").into());
                            container(inner).center_x(RIGHT_COLUMN_WIDTH).into()
                        });

                    let metadata_row = |description, cover| -> Element<'_, _> {
                        row![description]
                            .push(cover)
                            .spacing(theme.space.md)
                            .align_y(Vertical::Center)
                            .into()
                    };

                    let content: Element<'_, _> = match (volume_slider, cover) {
                        (None, None) => row![description, buttons]
                            .spacing(theme.space.md)
                            .align_y(Vertical::Center)
                            .into(),
                        (Some(v), cover) => {
                            let controls = row![v, buttons]
                                .spacing(theme.space.md)
                                .align_y(Vertical::Center);
                            column![metadata_row(description, cover), controls]
                                .spacing(theme.space.md)
                                .into()
                        }
                        (None, cover) => {
                            use iced::widget::space;
                            let controls =
                                row![space::horizontal().width(LEFT_COLUMN_WIDTH), buttons]
                                    .spacing(theme.space.md)
                                    .align_y(Vertical::Center);
                            column![metadata_row(description, cover), controls]
                                .spacing(theme.space.md)
                                .into()
                        }
                    };

                    container(content)
                        .style(move |app_theme: &Theme| container::Style {
                            background: Background::Color(
                                app_theme
                                    .extended_palette()
                                    .background
                                    .weak
                                    .color
                                    .scale_alpha(theme.opacity),
                            )
                            .into(),
                            border: Border::default().rounded(theme.radius.lg),
                            ..container::Style::default()
                        })
                        .padding(theme.space.md)
                        .width(Length::Fill)
                        .into()
                }))
                .spacing(theme.space.md)
            )
            .spacing(theme.space.xs)
            .into(),
        })
        .width(MenuSize::Large)
        .into()
    }

    fn handle_command(&mut self, service_name: String, command: PlayerCommand) -> Task<Message> {
        match self.service.as_mut() {
            Some(s) => s
                .command(MprisPlayerCommand {
                    service_name,
                    command,
                })
                .map(Message::Event),
            _ => Task::none(),
        }
    }

    fn get_title(&self, d: &MprisPlayerData) -> String {
        match &d.metadata {
            Some(m) => truncate_text(&m.to_string(), self.config.max_title_length),
            None => "No Title".to_string(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        MprisPlayerService::subscribe().map(Message::Event)
    }
}
