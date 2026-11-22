use crate::{
    lang,
    model::{self, Answer, Record},
};
use sp_social::runtime::Names;
use strawberry_plugin_api::{
    Context, Plugin, PluginMetadata, Result, Server,
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    common::NamedColor,
    permission::{Permission, PermissionDefault, PermissionLevel},
    register_plugin,
    text::TextComponent,
};

const USE: &str = "seen-lite:use";

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

struct Run;

impl CommandHandler for Run {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> std::result::Result<i32, CommandError> {
        let viewer = sender.as_player();
        let locale = match &viewer {
            Some(player) => player.get_locale(),
            None => format!("{:?}", sender.get_locale()),
        };
        let refuse = |key: &str| {
            CommandError::CommandFailed(
                TextComponent::text(&lang::text(&locale, key, &[])).color_named(NamedColor::Red),
            )
        };
        if viewer.as_ref().is_some_and(|p| !p.has_permission(USE)) {
            return Err(refuse("no_permission"));
        }
        let raw = match args.get_value("player") {
            Arg::Simple(raw) => raw,
            _ => String::new(),
        };
        let name = model::name(&raw).ok_or_else(|| refuse("bad_name"))?;
        // The server's own record of who joined, answered from its memory: one call, no file.
        let record = server.get_offline_player_by_name(name);
        // Online and visible to the reader: a hidden staff member is answered as offline. The console
        // sees everybody.
        let online_and_visible = record.as_ref().is_some_and(|record| {
            server
                .get_player_by_uuid(record.id)
                .is_some_and(|online| match &viewer {
                    Some(viewer) => viewer.can_see_player(online),
                    None => true,
                })
        });
        let record = record.map(|record| Record {
            name: record.name,
            last_seen_ms: record.last_seen,
        });
        let line = match model::answer(record, online_and_visible, now_ms()) {
            Answer::Online { name, for_ms } => {
                let time = lang::span(&locale, &model::span(for_ms));
                TextComponent::text(&lang::text(&locale, "online", &[&name, &time]))
                    .color_named(NamedColor::Green)
            }
            Answer::Offline { name, ago_ms } => {
                let time = lang::span(&locale, &model::span(ago_ms));
                TextComponent::text(&lang::text(&locale, "offline", &[&name, &time]))
                    .color_named(NamedColor::Gold)
            }
            Answer::Never => TextComponent::text(&lang::text(&locale, "never", &[name]))
                .color_named(NamedColor::Gray),
        };
        sender.send_message(line);
        Ok(1)
    }
}

struct SeenLite;

impl Plugin for SeenLite {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "seen-lite".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Strawberry Interactive LLC".into()],
            description: lang::text("en", "description", &[]),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        // EssentialsX's `essentials.seen` is for operators by default; so is this.
        context.register_permission(&Permission {
            node: USE.into(),
            description: lang::text("en", "permission_use", &[]),
            default: PermissionDefault::Op(PermissionLevel::Two),
            children: vec![],
        })?;
        context.register_command(
            Command::new(&["seen".into()], &lang::text("en", "command_seen", &[])).then(
                CommandNode::argument("player", &ArgumentType::String(StringType::SingleWord))
                    .suggest(Names)
                    .execute(Run),
            ),
            USE,
        );
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        Ok(())
    }
}

register_plugin!(SeenLite);
