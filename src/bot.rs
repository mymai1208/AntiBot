use std::sync::Arc;

use poise::{
    builtins,
    serenity_prelude::{
        self, interaction, GatewayIntents, Interaction,
    },
    Event, Framework, FrameworkOptions,
};

use crate::{CONFIG, KEY_MANAGER};

pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn create_bot(
    token: &str,
) -> Result<Arc<Framework<Data, Error>>, Box<dyn std::error::Error>> {
    let intents: GatewayIntents =
        GatewayIntents::GUILD_MEMBERS | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

    Ok(Framework::builder()
        .intents(intents)
        .token(token)
        .options(FrameworkOptions {
            commands: vec![setup()],
            event_handler: |ctx, event, _framework, _data| {
                Box::pin(async move {
                    on_event(ctx, event).await?;

                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        })
        .build()
        .await?)
}

#[poise::command(slash_command)]
async fn setup(
    ctx: Context<'_>,
    #[description = "role"] role: serenity_prelude::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    ctx.channel_id()
        .send_message(ctx.http(), |m| {
            m.content("入室するにはボタンをクリックして、表示されるリンクを開いてください。")
                .components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.custom_id("verify")
                                .label("Verify")
                                .style(serenity_prelude::ButtonStyle::Primary)
                        })
                    })
                })
        })
        .await?;

    CONFIG
        .lock()
        .unwrap()
        .update_server_config(*guild_id.as_u64(), *role.id.as_u64())?;

    Ok(())
}

async fn on_event(
    ctx: &serenity_prelude::Context,
    event: &Event<'_>
) -> Result<(), Error> {
    match event {
        Event::InteractionCreate { interaction } => {
            if let Interaction::MessageComponent(component) = interaction {
                if component.data.custom_id != "verify" {
                    return Ok(());
                }

                component
                    .create_interaction_response(&ctx.http, |r| {
                        r.kind(interaction::InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|d| {
                                let url = format!(
                                    "http://localhost:3000/verify/{}",
                                    KEY_MANAGER.lock().unwrap().create_key(
                                        *component.guild_id.unwrap().as_u64(),
                                        *component.user.id.as_u64()
                                    )
                                );

                                d.ephemeral(true).content(url)
                            })
                    })
                    .await
                    .unwrap();
            }
        }
        _ => {}
    }

    Ok(())
}
