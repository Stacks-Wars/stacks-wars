import type { BetterAuthClientPlugin } from "better-auth";
import type { siws } from ".";

type SiwsPlugin = typeof siws;

export const siwsClient = () => {
	return {
		id: "siws",
		$InferServerPlugin: {} as ReturnType<SiwsPlugin>,
	} satisfies BetterAuthClientPlugin;
};
