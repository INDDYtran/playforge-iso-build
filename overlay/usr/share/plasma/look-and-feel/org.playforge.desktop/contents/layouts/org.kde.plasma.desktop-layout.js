loadTemplate("org.kde.plasma.desktop.defaultPanel");

var desktopsArray = desktopsForActivity(currentActivity());
for (var j = 0; j < desktopsArray.length; j++) {
    desktopsArray[j].wallpaperPlugin = 'org.kde.image';
    try {
        desktopsArray[j].currentConfigGroup = ['Wallpaper', 'org.kde.image', 'General'];
        desktopsArray[j].writeConfig('Image', 'file:///usr/share/wallpapers/Playforge/contents/images/2560x1440.png');
        desktopsArray[j].writeConfig('FillMode', 2);
    } catch (e) {
        // Das Wallpaper wird zusätzlich beim ersten Login gesetzt.
    }
}
