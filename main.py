import asyncio
import os
import pickle
import tomllib

import discord
from aiohttp import web
from discord.ext import commands
from pretty_help import PrettyHelp

with open("config.toml", "rb") as f:
    config = tomllib.load(f)


async def main():
    # start the client
    async with client:
        for filename in os.listdir("./cogs"):
            if filename.endswith(".py"):
                await client.load_extension(f"cogs.{filename[:-3]}")
        await client.start(config["general"]["token"])


class Bot(commands.Bot):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.app = web.Application()
        self.app.router.add_post("/post", self.inventory)
        self.app.router.add_get("/get", self.get_instructions)
        self.app.router.add_route("OPTIONS", "/get", self.handle_options)
        self.app.router.add_route("OPTIONS", "/post", self.handle_options)
        self.runner = web.AppRunner(self.app)

    async def handle_options(self, request):
        # do nothing, just respond with OK
        return web.Response(
            text="Options received",
            headers={
                "Access-Control-Allow-Origin": "*",
                "Access-Control-Allow-Methods": "GET,POST,OPTIONS",
                "Access-Control-Allow-Headers": "Content-Type",
            },
        )

    async def get_instructions(self, request): ...
    async def inventory(self, request): ...


intents = discord.Intents().all()
client = Bot(
    command_prefix=config["general"]["prefix"],
    intents=intents,
    help_command=PrettyHelp(),
)


# @client.command(hidden=True)
# async def load(ctx, extension):
#     if ctx.author.id == config["general"]["owner_id"]:
#         await client.load_extension(f"cogs.{extension}")
#         await ctx.send(f"{extension} loaded.")
#     if ctx.author.id != config["general"]["owner_id"]:
#         await ctx.send("no")


# @client.command(hidden=True)
# async def unload(ctx, extension):
#     if ctx.author.id == config:
#         await client.unload_extension(f"cogs.{extension}")
#         await ctx.send(f"{extension} unloaded.")

#     if ctx.author.id != config["general"]["owner_id"]:
#         await ctx.send("no")


# @client.command(hidden=True)
# async def reload(ctx, extension):
#     if ctx.author.id == config["general"]["owner_id"]:
#         await client.unload_extension(f"cogs.{extension}")
#         await ctx.send(f"{extension} unloaded.")
#         await client.load_extension(f"cogs.{extension}")
#         await ctx.send(f"{extension} loaded.")
#     else:
#         await ctx.send("no")

data = {}

# Assign values
data[-100, 20, -25] = "thing"
data[0, 0, 0] = "origin"

# Save
with open("data/3d_data.pkl", "wb") as f:
    pickle.dump(data, f)

# Later, load it
with open("data/3d_data.pkl", "rb") as f:
    data = pickle.load(f)

print(data[-100, 20, -25])  # Output: "thing"


@client.event
async def on_ready():
    print("I am ready.")
    try:
        synced = await client.tree.sync()
        print(f"Sycned {len(synced)} commands!")
    except Exception as e:
        print(e)


if __name__ == "__main__":
    discord.utils.setup_logging()
    asyncio.run(main())
