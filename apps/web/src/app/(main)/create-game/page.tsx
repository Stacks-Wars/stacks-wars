import CreateGameForm from "@/components/main/create-game-form";

export default function CreateGamePage() {
	return (
		<div className="container mx-auto px-4 py-8 sm:py-12">
			<div className="mx-auto max-w-2xl">
				<div className="mb-6 sm:mb-8">
					<h1 className="mb-2 text-2xl font-bold sm:text-4xl">
						Create New Game
					</h1>
					<p className="text-muted-foreground text-sm sm:text-base">
						Add a new game type to the platform
					</p>
				</div>
				<div className="bg-card rounded-4xl p-4 sm:p-8">
					<CreateGameForm />
				</div>
			</div>
		</div>
	);
}
