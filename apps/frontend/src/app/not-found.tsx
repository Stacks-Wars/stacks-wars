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
		<div className="min-h-screen flex items-center justify-center bg-gradient-to-b from-background to-primary/30">
			<div className="text-center space-y-8 px-4">
				{/* Error Code */}
				<div className="space-y-2">
					<h1 className="text-8xl font-bold text-primary/20">404</h1>
					<h2 className="text-2xl sm:text-3xl font-bold text-foreground">
						{page ? page : "Page Not Found or Lobby does not exist"}
					</h2>
					<p className="text-muted-foreground max-w-md mx-auto">
						Sorry, we couldn&apos;t find the page you&apos;re
						looking for. It might have been moved, deleted, or
						doesn&apos;t exist.
					</p>
				</div>

				<div className="flex flex-col sm:flex-row gap-3 justify-center items-center">
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

				<div className="pt-4 border-t border-primary/20">
					<p className="text-sm text-muted-foreground">
						If you believe this is an error, please contact support
						or try refreshing the page.
					</p>
				</div>
			</div>
		</div>
	);
}
