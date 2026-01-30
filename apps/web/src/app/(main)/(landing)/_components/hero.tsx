import Image from "next/image";

export default function Hero() {
	return (
		<div className="flex items-center justify-center py-52 px-14 w-full bg-[#312F2D] flex-col gap-3 rounded-4xl">
			<h3 className="capitalize text-8xl font-bold">battle of words</h3>
			<p className="font-medium text-2xl max-w-3xl text-center px-4">
				Experience the thrill of Stacks Wars with our first game. Dive in, test
				your skills, and claim victory!
			</p>
			<Image className="w-32 h-32 " src={"/shapes/Rectangle.png"} />
		</div>
	);
}
