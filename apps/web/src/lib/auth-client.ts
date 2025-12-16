import { siwsClient } from "@stacks-wars/auth/plugins/siws/client";
import { BETTERAUTH_URL } from "@stacks-wars/shared";
import { createAuthClient } from "better-auth/react";

export const authClient = createAuthClient({
	baseURL: BETTERAUTH_URL,
	plugins: [siwsClient()],
});
