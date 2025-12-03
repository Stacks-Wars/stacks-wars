import KeyboardReact from "react-simple-keyboard";
import "react-simple-keyboard/build/css/index.css";
import "./keyboard-theme.css";

const layout = {
	default: [
		"q w e r t y u i o p",
		"a s d f g h j k l",
		"{shift} z x c v b n m {bksp}",
		"{enter}",
	],
	shift: [
		"Q W E R T Y U I O P",
		"A S D F G H J K L",
		"{shift} Z X C V B N M {bksp}",
		"{enter}",
	],
};

const display = {
	"{bksp}": "⌫",
	"{enter}": "Submit",
	"{shift}": "⇧",
};

interface KeyboardProps {
	onKeyPress: (key: string) => void;
	layoutName: string;
}

export default function Keyboard({ onKeyPress, layoutName }: KeyboardProps) {
	return (
		<div className="fixed right-0 bottom-0 left-0 z-50">
			<div className="hg-theme-default mx-auto max-w-3xl">
				<KeyboardReact
					layout={layout}
					layoutName={layoutName}
					display={display}
					mergeDisplay={true}
					onKeyPress={onKeyPress}
				/>
			</div>
		</div>
	);
}
