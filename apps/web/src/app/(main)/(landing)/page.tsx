import GamesShowcase from "./_components/games-showcase";
import Hero from "./_components/hero";
import PlayToEarn from "./_components/play-to-earn";
import TryOut from "./_components/try-out";

export default function HomePage() {
	return (
		<div className="container mx-auto px-4 py-8 !overflow-x-hidden">
			<Hero />
			<GamesShowcase />
			<PlayToEarn />
			<TryOut />
		</div>
	);
}
