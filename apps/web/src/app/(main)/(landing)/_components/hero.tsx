import Image from "next/image";
import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function Hero() {
	return (
		<div className="relative w-full">
			<div className="relative flex min-h-[50vh] w-full flex-col items-center justify-center gap-6 overflow-hidden rounded-3xl bg-[#312F2D] px-4 py-16 md:rounded-4xl lg:min-h-[55vh] lg:gap-8 lg:py-20">
				{/* Hero Banner Background */}
				<Image
					className="absolute inset-0 z-0 h-full w-full object-cover"
					src="/images/hero-banner.png"
					fill
					sizes="100vw"
					alt=""
					priority
				/>

				{/* Optional overlay for better text readability */}
				<div className="absolute inset-0 z-0 bg-black/40" />

				{/* Content */}
				<div className="relative z-10 flex flex-col items-center gap-4 lg:gap-6">
					<h1 className="text-center text-4xl font-bold text-white md:text-6xl lg:text-8xl">
						<span className="block">Stacks Wars</span>
						<span className="block">X</span>
						<span className="block md:inline"> DeGrant</span>
					</h1>
					<p className="max-w-3xl px-4 text-center text-base font-medium text-white/90 md:text-xl lg:text-2xl">
						Experience the thrill of Stacks Wars with our first
						game. Dive in, test your skills, and claim victory!
					</p>
				</div>

				{/* Ruby Images */}
				<Image
					className="absolute -top-8 left-[31.33px] z-10 h-20 w-20 shrink-0 scale-x-[-1] object-contain min-[500px]:-top-16 min-[500px]:h-32 min-[500px]:w-32 md:h-56 md:w-56 lg:h-72 lg:w-72"
					src="/images/ruby1.png"
					width={288}
					height={288}
					sizes="(max-width: 500px) 80px, (max-width: 768px) 128px, (max-width: 1024px) 224px, 288px"
					alt=""
				/>
				<Image
					className="absolute top-[50%] -right-4 z-10 h-20 w-20 shrink-0 object-contain min-[500px]:-right-8 min-[500px]:h-32 min-[500px]:w-32 md:-right-12 md:h-56 md:w-56 lg:-right-16 lg:h-72 lg:w-72"
					src="/images/ruby1.png"
					width={288}
					height={288}
					sizes="(max-width: 500px) 80px, (max-width: 768px) 128px, (max-width: 1024px) 224px, 288px"
					alt=""
				/>
			</div>

			{/* Button - Outside the overflow-hidden container */}
			<div className="relative z-20 -mt-6 flex justify-center sm:-mt-7 lg:-mt-10">
				<Button
					variant="outline"
					className="h-12 w-full max-w-52 rounded-full px-8 py-3 text-base font-semibold shadow-lg sm:h-14 sm:max-w-64 sm:text-lg lg:h-16 lg:max-w-80 lg:text-xl"
					asChild
				>
					<Link href="/games">Play Now!</Link>
				</Button>
			</div>
		</div>
	);
}
