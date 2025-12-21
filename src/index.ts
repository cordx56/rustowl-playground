import { Container, getContainer } from "@cloudflare/containers";
import { Hono } from "hono";

export class RustOwlContainer extends Container<Env> {
  // Port the container listens on (default: 8080)
  defaultPort = 3000;
  // Time before container sleeps due to inactivity (default: 30s)
  sleepAfter = "2m";
  // Environment variables passed to the container
  envVars = {};

  // Optional lifecycle hooks
  override onStart() {
    console.log("Container successfully started");
  }

  override onStop() {
    console.log("Container successfully shut down");
  }

  override onError(error: unknown) {
    console.log("Container error:", error);
  }
}

// Create Hono app with proper typing for Cloudflare Workers
const app = new Hono<{
  Bindings: Env;
}>();

app.post("/api/analyze", async (c) => {
  const container = getContainer(c.env.RUSTOWL_CONTAINER);
  return await container.fetch(c.req.raw);
});

// Home route with available endpoints
app.get("*", async (c) => {
  return await c.env.ASSETS.fetch(c.req.raw);
});

export default app;
