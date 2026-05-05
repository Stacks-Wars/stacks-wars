"use client";

import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { getQueryClient } from "./get-query-client";
import { AuthProvider } from "@/lib/contexts/auth-context";
import { AudioProvider } from "@/components/main/audio-provider";

export function Provider({ children }: { children: React.ReactNode }) {
	const queryClient = getQueryClient();

	return (
		<QueryClientProvider client={queryClient}>
			<AuthProvider>
				<AudioProvider>{children}</AudioProvider>
			</AuthProvider>
			<ReactQueryDevtools initialIsOpen={false} />
		</QueryClientProvider>
	);
}
