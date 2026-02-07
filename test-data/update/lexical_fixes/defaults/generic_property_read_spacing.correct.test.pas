unit GenericPropertyReadSpacing;

interface

type
  TMy = class
  private
    FGeom: TObject;
  public
    property Geom: TObjectList<TObject> read FGeom;
    property Geom2: TObjectList<TObject> read FGeom;
  end;

implementation

end.
