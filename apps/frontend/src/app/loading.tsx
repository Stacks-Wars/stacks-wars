import Image from "next/image";

export default function Loading() {
	return (
		<main className="from-background to-primary/30 min-h-screen bg-gradient-to-b">
			<div className="mx-auto max-w-3xl p-4 sm:p-6">
				<div className="flex min-h-[70vh] flex-col items-center justify-center space-y-8">
					<div className="animate-bounce">
						<Image
							src="/logo.webp"
							alt="Stacks Wars"
							width={200}
							height={200}
							className="h-32 w-32 rounded-md sm:h-40 sm:w-40 md:h-48 md:w-48"
						/>
					</div>

					<div className="flex space-x-2">
						<div className="bg-primary h-2 w-2 animate-pulse rounded-full"></div>
						<div className="bg-primary h-2 w-2 animate-pulse rounded-full delay-75"></div>
						<div className="bg-primary h-2 w-2 animate-pulse rounded-full delay-150"></div>
					</div>
				</div>
			</div>
		</main>
	);
}
