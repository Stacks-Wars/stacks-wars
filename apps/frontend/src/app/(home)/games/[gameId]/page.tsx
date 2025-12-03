import { ArrowLeft } from "lucide-react";
import Link from "next/link";
import CreateLobbyForm from "./_components/create-lobby-form";
import CreateSponsoredLobbyForm from "./_components/create-sponsored-lobby-form";
import GameDetails from "./_components/game-details";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { apiRequest } from "@/lib/api";
import { GameType } from "@/types/schema/game";

export default async function CreateGame({
	params,
}: {
	params: Promise<{ gameId: string }>;
}) {
	const { gameId } = await params;

	const game = await apiRequest<GameType>({
		path: `/game/${gameId}`,
		auth: false,
	});

	return (
		<div className="mx-auto max-w-3xl px-4 py-4 sm:px-6 sm:py-6">
			<Link
				href="/games"
				className="text-muted-foreground hover:text-foreground mb-4 inline-flex items-center gap-1 text-sm font-medium transition-colors sm:mb-6"
			>
				<ArrowLeft className="h-4 w-4" />
				<span>Back to Games</span>
			</Link>

			<div className="space-y-6 sm:space-y-8">
				<GameDetails game={game} />

				<Tabs defaultValue="normal" className="w-full">
					<TabsList className="bg-primary/30 grid w-full grid-cols-2">
						<TabsTrigger value="normal">Normal</TabsTrigger>
						<TabsTrigger value="sponsored">Sponsored</TabsTrigger>
					</TabsList>

					<TabsContent value="normal" className="space-y-6">
						<CreateLobbyForm gameId={gameId} />
					</TabsContent>

					<TabsContent value="sponsored" className="space-y-6">
						<CreateSponsoredLobbyForm gameId={gameId} />
					</TabsContent>
				</Tabs>
			</div>
		</div>
	);
}
