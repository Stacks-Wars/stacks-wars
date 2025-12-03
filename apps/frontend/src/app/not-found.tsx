"use client";

import { Button } from "@/components/ui/button";
import { RefreshCw, ArrowLeft, Home } from "lucide-react";
import { useRouter } from "next/navigation";

export default function NotFound({ page }: { page?: string }) {
	const router = useRouter();

	const handleRefresh = () => {
		router.refresh();
	};

	const handleGoBack = () => {
		router.back();
	};

	const handleGoHome = () => {
		router.push("/");
	};

	return (
		<div className="from-background to-primary/30 flex min-h-screen items-center justify-center bg-gradient-to-b">
			<div className="space-y-8 px-4 text-center">
				{/* Error Code */}
				<div className="space-y-2">
					<h1 className="text-primary/20 text-8xl font-bold">404</h1>
					<h2 className="text-foreground text-2xl font-bold sm:text-3xl">
						{page ? page : "Page Not Found or Lobby does not exist"}
					</h2>
					<p className="text-muted-foreground mx-auto max-w-md">
						Sorry, we couldn&apos;t find the page you&apos;re
						looking for. It might have been moved, deleted, or
						doesn&apos;t exist.
					</p>
				</div>

				<div className="flex flex-col items-center justify-center gap-3 sm:flex-row">
					<Button
						onClick={handleGoBack}
						variant="outline"
						className="w-full sm:w-auto"
					>
						<ArrowLeft className="mr-2 h-4 w-4" />
						Go Back
					</Button>

					<Button
						onClick={handleRefresh}
						variant="outline"
						className="w-full sm:w-auto"
					>
						<RefreshCw className="mr-2 h-4 w-4" />
						Refresh Page
					</Button>

					<Button onClick={handleGoHome} className="w-full sm:w-auto">
						<Home className="mr-2 h-4 w-4" />
						Go Home
					</Button>
				</div>

				<div className="border-primary/20 border-t pt-4">
					<p className="text-muted-foreground text-sm">
						If you believe this is an error, please contact support
						or try refreshing the page.
					</p>
				</div>
			</div>
		</div>
	);
}
