pub const CARD_STYLE: &str = "
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
    border-radius: 12px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    cursor: pointer;
    transition: transform 0.15s ease-in-out;
";
pub const CONTAINER_STYLE: &str = "
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    height: 100%;
    box-sizing: border-box;
";

pub const HEADER_STYLE: &str = "
    margin-bottom: 16px;
    text-align: center;
    font-weight: 700;
    font-size: 1.75rem;
";

pub const GRID_STYLE: &str = "
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 24px;
    width: 100%;
";
pub const NAME_STYLE: &str = "margin-top: 8px; text-align: center; font-weight: bold;";

pub const LOGIN_CONTAINER: &str = "
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    padding: 24px;
";

pub const LOGIN_FORM: &str = "
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 300px;
";

pub const LOGIN_ERROR: &str = "
    color: red;
    font-size: 0.9em;
";

pub const LOGIN_BUTTON: &str = "
    bg-blue-600
    hover:bg-blue-700
    active:bg-blue-800
    text-white
    font-semibold
    py-2
    px-4
    rounded-lg
    transition-colors
    duration-150
    disabled:opacity-50
    disabled:cursor-not-allowed
";

pub const TOPBAR: &str = "
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background-color: #1d273eff;
    width: 100%;
    position: sticky;
    top: 0;
    left: 0;
    box-sizing: border-box;
    z-index: 100;
";

pub const CHAPTER_BUTTON: &str="
    all: unset;
    box-sizing: border-box;
    display: block;
    width: 100%;
    padding: 6px 8px 6px 16px;

    cursor: pointer;
    white-space: normal;
    word-break: break-word;
    overflow-wrap: anywhere;
";
