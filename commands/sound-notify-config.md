---
name: sound-notify-config
description: Configure the AI Sound Notify server URL
---

Help the user configure the AI Sound Notify plugin.

## What to do

1. Check if `~/.claude/ai-sound-notify.local.md` already exists. If it does, read it and show the current `server_url` value.

2. Ask the user for their AI Sound Notify server URL. Common examples:
   - Local: `http://localhost:9800`
   - Remote: `https://ainotify.example.com`

3. Write the config file to `~/.claude/ai-sound-notify.local.md` with this format:

```
---
server_url: <the URL they provide>
---
# AI Sound Notify Configuration

Server URL configured for sound notifications.
Open the server URL in a browser to see and hear notifications.
```

4. Test the connection by running:
```bash
curl -s <server_url>/api/health
```

5. If the health check succeeds, send a test notification:
```bash
curl -s -X POST <server_url>/notify -H 'Content-Type: application/json' -d '{"source":"claude-code","event":"task_complete","message":"Plugin configured successfully!"}'
```

6. Report the result to the user. If it worked, tell them the plugin is ready - they should hear a sound in their browser.
