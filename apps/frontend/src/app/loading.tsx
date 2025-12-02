import Image from "next/image";

export default function Loading() {
	return (
		<main className="min-h-screen bg-gradient-to-b from-background to-primary/30">
			<div className="max-w-3xl mx-auto p-4 sm:p-6">
				<div className="min-h-[70vh] flex flex-col items-center justify-center space-y-8">
					<div className="animate-bounce">
						<Image
							src="/logo.webp"
							alt="Stacks Wars"
							width={200}
							height={200}
							className="w-32 h-32 sm:w-40 sm:h-40 md:w-48 md:h-48 rounded-md"
						/>
					</div>

					<div className="flex space-x-2">
						<div className="w-2 h-2 bg-primary rounded-full animate-pulse"></div>
						<div className="w-2 h-2 bg-primary rounded-full animate-pulse delay-75"></div>
						<div className="w-2 h-2 bg-primary rounded-full animate-pulse delay-150"></div>
					</div>
				</div>
			</div>
		</main>
	);
}
