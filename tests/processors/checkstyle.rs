test_checker!(checkstyle, tool: "checkstyle", processor: "checkstyle",
    config: "[processor]\nenabled = [\"checkstyle\"]\n\n[processor.checkstyle]\nargs = [\"-c\", \"/google_checks.xml\"]\n",
    files: [("Test.java", "public class Test {\n    public static void main(String[] args) {\n    }\n}\n")]);
