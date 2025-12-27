import CreateGameForm from "./_components/create-game-form";

export default function CreateGamePage() {
	return (
		<div className="container mx-auto px-4 py-8 sm:py-12">
			<div className="max-w-2xl mx-auto">
				<div className="mb-6 sm:mb-8">
					<h1 className="text-2xl sm:text-4xl font-bold mb-2">
						Create New Game
					</h1>
					<p className="text-sm sm:text-base text-muted-foreground">
						Add a new game type to the platform
					</p>
				</div>
				<div className="bg-card p-4 sm:p-8 rounded-4xl">
					<CreateGameForm />
				</div>
			</div>
		</div>
	);
}
