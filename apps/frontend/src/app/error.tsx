"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { RefreshCw, Home, Bug } from "lucide-react";

export default function GlobalError({
	error,
}: {
	error: Error;
	reset: () => void;
}) {
	const router = useRouter();

	useEffect(() => {
		console.error("🔥 Unhandled error:", error);
	}, [error]);

	const handleRefresh = () => {
		window.location.reload();
	};

	const handleGoHome = () => {
		router.push("/");
	};

	return (
		<div className="from-background to-primary/30 flex min-h-screen items-center justify-center bg-linear-to-b">
			<div className="space-y-8 px-4 text-center">
				{/* Error Code */}
				<div className="space-y-2">
					<h1 className="text-destructive/20 text-8xl font-bold">
						500
					</h1>
					<h2 className="text-foreground text-2xl font-bold sm:text-3xl">
						Something Went Wrong
					</h2>
					<p className="text-muted-foreground mx-auto max-w-md">
						An unexpected error occurred. We&apos;re working on it,
						but you can try refreshing or head back home.
					</p>
				</div>

				<div className="flex flex-col items-center justify-center gap-3 sm:flex-row">
					<Button
						onClick={handleRefresh}
						variant="outline"
						className="w-full sm:w-auto"
					>
						<RefreshCw className="mr-2 h-4 w-4" />
						Retry
					</Button>

					<Button onClick={handleGoHome} className="w-full sm:w-auto">
						<Home className="mr-2 h-4 w-4" />
						Go Home
					</Button>
				</div>

				<div className="border-destructive/30 border-t pt-4">
					<p className="text-muted-foreground text-sm">
						If the problem persists, please report it or check back
						later. <Bug className="ml-1 inline h-4 w-4" />
					</p>
				</div>
			</div>
		</div>
	);
}
